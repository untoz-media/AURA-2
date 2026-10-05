param(
  [switch]$ApplyEnvironment
)

$ErrorActionPreference = "Stop"

if ($env:OS -ne "Windows_NT") {
  throw "AURA-2 Windows Build Doctor must run on Windows."
}

$script:Blockers = New-Object System.Collections.Generic.List[string]
$script:Warnings = New-Object System.Collections.Generic.List[string]
$script:VsDevCmd = $null
$script:MsvcLink = $null
$script:MsvcCl = $null

function Write-DoctorResult {
  param(
    [Parameter(Mandatory = $true)][string]$State,
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$Detail
  )

  $prefix = switch ($State) {
    "PASS" { "[PASS]" }
    "WARN" { "[WARN]" }
    default { "[FAIL]" }
  }
  Write-Host ("{0} {1} — {2}" -f $prefix, $Name, $Detail)
}

function Add-Blocker {
  param([string]$Name, [string]$Detail)
  $script:Blockers.Add(("{0}: {1}" -f $Name, $Detail))
  Write-DoctorResult "FAIL" $Name $Detail
}

function Add-Warning {
  param([string]$Name, [string]$Detail)
  $script:Warnings.Add(("{0}: {1}" -f $Name, $Detail))
  Write-DoctorResult "WARN" $Name $Detail
}

function Find-VisualStudioInstallation {
  $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
  if (Test-Path $vswhere) {
    $path = (& $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null | Select-Object -First 1)
    if ($path -and (Test-Path $path)) {
      return $path.Trim()
    }
  }

  $root = Join-Path ${env:ProgramFiles} "Microsoft Visual Studio\2022"
  foreach ($edition in @("BuildTools", "Community", "Professional", "Enterprise")) {
    $candidate = Join-Path $root $edition
    if (Test-Path $candidate) {
      return $candidate
    }
  }

  return $null
}

function Import-VsDevEnvironment {
  param([Parameter(Mandatory = $true)][string]$VsDevCmd)

  $command = "call `"$VsDevCmd`" -no_logo -arch=x64 -host_arch=x64 && set"
  $lines = & $env:ComSpec /d /c $command
  if ($LASTEXITCODE -ne 0) {
    throw "Visual Studio developer environment exited with code $LASTEXITCODE."
  }

  foreach ($line in $lines) {
    if ($line -match "^([^=]+)=(.*)$") {
      [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
    }
  }
}

Write-Host ""
Write-Host "AURA-2 Windows Build Doctor"
Write-Host "==========================="
Write-Host ""

$psMajor = $PSVersionTable.PSVersion.Major
if ($psMajor -ge 5) {
  Write-DoctorResult "PASS" "PowerShell" $PSVersionTable.PSVersion.ToString()
} else {
  Add-Blocker "PowerShell" "PowerShell 5.1 or newer is required."
}

$git = Get-Command git.exe -ErrorAction SilentlyContinue
if ($git) {
  $gitVersion = (& git --version).Trim()
  Write-DoctorResult "PASS" "Git" $gitVersion
} else {
  Add-Blocker "Git" "git.exe was not found on PATH."
}

$node = Get-Command node.exe -ErrorAction SilentlyContinue
if ($node) {
  $nodeVersion = (& node --version).Trim()
  $nodeMajor = [int](($nodeVersion -replace "^v", "").Split(".")[0])
  if ($nodeMajor -ge 22) {
    Write-DoctorResult "PASS" "Node.js" $nodeVersion
  } else {
    Add-Blocker "Node.js" "$nodeVersion is installed; AURA-2 Beta builds require Node.js 22 or newer."
  }
} else {
  Add-Blocker "Node.js" "node.exe was not found on PATH. Install Node.js 22 LTS or newer."
}

$npm = Get-Command npm.cmd -ErrorAction SilentlyContinue
if ($npm) {
  Write-DoctorResult "PASS" "npm" ((& npm --version).Trim())
} else {
  Add-Blocker "npm" "npm.cmd was not found on PATH."
}

$cargo = Get-Command cargo.exe -ErrorAction SilentlyContinue
$rustc = Get-Command rustc.exe -ErrorAction SilentlyContinue
if ($cargo -and $rustc) {
  $rustVersion = (& rustc --version).Trim()
  $hostLine = (& rustc -vV | Select-String "^host:" | Select-Object -First 1).ToString()
  $host = ($hostLine -replace "^host:\s*", "").Trim()
  if ($host -eq "x86_64-pc-windows-msvc") {
    Write-DoctorResult "PASS" "Rust MSVC toolchain" "$rustVersion · $host"
  } else {
    Add-Blocker "Rust MSVC toolchain" "Detected host '$host'. Install/select stable-x86_64-pc-windows-msvc."
  }
} else {
  Add-Blocker "Rust" "cargo.exe/rustc.exe were not found. Install Rust with the MSVC host toolchain."
}

$vsInstall = Find-VisualStudioInstallation
if ($vsInstall) {
  $script:VsDevCmd = Join-Path $vsInstall "Common7\Tools\VsDevCmd.bat"
  $msvcRoots = Get-ChildItem (Join-Path $vsInstall "VC\Tools\MSVC") -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending
  foreach ($root in $msvcRoots) {
    $link = Join-Path $root.FullName "bin\Hostx64\x64\link.exe"
    $cl = Join-Path $root.FullName "bin\Hostx64\x64\cl.exe"
    if ((Test-Path $link) -and (Test-Path $cl)) {
      $script:MsvcLink = $link
      $script:MsvcCl = $cl
      break
    }
  }

  if ($script:MsvcLink) {
    Write-DoctorResult "PASS" "Visual Studio C++ Build Tools" $vsInstall
    Write-DoctorResult "PASS" "MSVC linker" $script:MsvcLink
  } else {
    Add-Blocker "MSVC linker" "Visual Studio is present, but the x64 C++ toolchain/link.exe was not found. Add 'Desktop development with C++' and the MSVC v143 tools."
  }

  if (-not (Test-Path $script:VsDevCmd)) {
    Add-Warning "Visual Studio environment" "VsDevCmd.bat was not found at the expected path."
    $script:VsDevCmd = $null
  }
} else {
  Add-Blocker "Visual Studio C++ Build Tools" "No Visual Studio 2022 installation with C++ tools was detected. Install Visual Studio Build Tools 2022 with 'Desktop development with C++'."
}

$windowsKitsRoot = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\Lib"
$sdkLib = $null
if (Test-Path $windowsKitsRoot) {
  $sdkLib = Get-ChildItem $windowsKitsRoot -Directory -ErrorAction SilentlyContinue |
    Sort-Object Name -Descending |
    ForEach-Object {
      $kernel32 = Join-Path $_.FullName "um\x64\kernel32.lib"
      if (Test-Path $kernel32) { $kernel32 }
    } |
    Select-Object -First 1
}
if ($sdkLib) {
  Write-DoctorResult "PASS" "Windows SDK" $sdkLib
} else {
  Add-Blocker "Windows SDK" "Windows 10/11 SDK x64 libraries were not found. Add a Windows SDK in Visual Studio Installer."
}

$pythonDetail = $null
$pythonOk = $false
$python = Get-Command python.exe -ErrorAction SilentlyContinue
if ($python) {
  try {
    $pythonDetail = (& python --version 2>&1).ToString().Trim()
    if ($LASTEXITCODE -eq 0) { $pythonOk = $true }
  } catch {}
}
if (-not $pythonOk) {
  $py = Get-Command py.exe -ErrorAction SilentlyContinue
  if ($py) {
    try {
      $pythonDetail = (& py -3.12 --version 2>&1).ToString().Trim()
      if ($LASTEXITCODE -eq 0) { $pythonOk = $true }
    } catch {}
  }
}
if ($pythonOk) {
  Write-DoctorResult "PASS" "Python build validation" $pythonDetail
} else {
  Add-Blocker "Python build validation" "Python 3 was not found. Install Python 3.12 or make a working python.exe/py -3.12 available."
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$root = [System.IO.Path]::GetPathRoot($repoRoot)
$driveName = $root.TrimEnd("\").TrimEnd(":")
$drive = Get-PSDrive -Name $driveName -ErrorAction SilentlyContinue
if ($drive) {
  $freeGb = [math]::Round($drive.Free / 1GB, 1)
  if ($drive.Free -ge 5GB) {
    Write-DoctorResult "PASS" "Build disk space" "$freeGb GB free on $root"
  } else {
    Add-Blocker "Build disk space" "Only $freeGb GB is free on $root; keep at least 5 GB available for dependencies and build artifacts."
  }
} else {
  Add-Warning "Build disk space" "Could not inspect free space for $root."
}

$webViewRoots = @(
  (Join-Path ${env:ProgramFiles(x86)} "Microsoft\EdgeWebView\Application"),
  (Join-Path ${env:ProgramFiles} "Microsoft\EdgeWebView\Application")
)
$webView = $webViewRoots | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($webView) {
  Write-DoctorResult "PASS" "WebView2 Runtime" $webView
} else {
  Add-Warning "WebView2 Runtime" "WebView2 was not detected in the standard machine-wide locations. The installer/runtime test should verify it before release."
}

if ($ApplyEnvironment -and $script:VsDevCmd) {
  try {
    Import-VsDevEnvironment $script:VsDevCmd
    $linkOnPath = Get-Command link.exe -ErrorAction SilentlyContinue
    if ($linkOnPath) {
      Write-DoctorResult "PASS" "MSVC environment" "Visual Studio developer environment loaded into this PowerShell process."
    } else {
      Add-Blocker "MSVC environment" "VsDevCmd.bat ran, but link.exe is still unavailable on PATH."
    }
  } catch {
    Add-Blocker "MSVC environment" $_.Exception.Message
  }
}

Write-Host ""
Write-Host "Build Doctor summary"
Write-Host "--------------------"
Write-Host ("Blockers: {0}" -f $script:Blockers.Count)
Write-Host ("Warnings: {0}" -f $script:Warnings.Count)

if ($script:Warnings.Count -gt 0) {
  foreach ($warning in $script:Warnings) {
    Write-Host ("WARN: {0}" -f $warning)
  }
}

if ($script:Blockers.Count -gt 0) {
  Write-Host ""
  Write-Host "AURA-2 cannot produce a trustworthy Windows build until the blockers above are fixed."
  Write-Host "For the MSVC blocker, install Visual Studio Build Tools 2022 and select:"
  Write-Host "  - Desktop development with C++"
  Write-Host "  - MSVC v143 x64/x86 build tools"
  Write-Host "  - Windows 10 or Windows 11 SDK"
  throw "Windows Build Doctor found $($script:Blockers.Count) blocking prerequisite(s)."
}

Write-Host ""
Write-Host "AURA-2 Windows build prerequisites look ready."
