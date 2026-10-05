$ErrorActionPreference = "Stop"

if ($env:OS -ne "Windows_NT") {
  throw "This script builds the AURA-2 Windows Beta installer and must run on Windows."
}

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

. (Join-Path $PSScriptRoot "windows-build-doctor.ps1") -ApplyEnvironment

$gitSha = (git rev-parse HEAD).Trim()
if (-not $gitSha) {
  throw "Could not resolve the source commit for this Beta build."
}
$env:AURA_BUILD_COMMIT = $gitSha
$env:AURA_BUILD_SOURCE = "local-beta-build"
$env:AURA_BUILD_LABEL = "beta-local-smoke"

$version = (Get-Content "package.json" -Raw | ConvertFrom-Json).version
$artifactDir = Join-Path $repoRoot "artifacts\beta\$version"

function Invoke-BetaStep {
  param(
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][scriptblock]$Command
  )

  Write-Host ""
  Write-Host "==> $Name"
  & $Command
  if ($LASTEXITCODE -ne 0) {
    throw "$Name failed with exit code $LASTEXITCODE."
  }
}

Invoke-BetaStep "Install JavaScript dependencies" { npm install }
Invoke-BetaStep "Validate Beta source" { npm run beta:validate }
Invoke-BetaStep "Run Rust regression tests" {
  cargo test --manifest-path "apps/desktop/src-tauri/Cargo.toml"
}
Invoke-BetaStep "Build Tauri + NSIS installer" { npm run build }

$installer = Get-ChildItem "apps/desktop/src-tauri/target/release/bundle/nsis/*.exe" |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1

if (-not $installer) {
  throw "NSIS installer was not produced."
}

New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

$copiedInstaller = Join-Path $artifactDir $installer.Name
Copy-Item $installer.FullName $copiedInstaller -Force

$hash = (Get-FileHash $copiedInstaller -Algorithm SHA256).Hash.ToLower()
$checksumPath = Join-Path $artifactDir "AURA-2-Windows-x64.sha256"
"$hash  $($installer.Name)" | Set-Content $checksumPath -Encoding ascii

$signature = Get-AuthenticodeSignature $copiedInstaller

$manifest = [ordered]@{
  schemaVersion = 2
  product = "AURA-2"
  version = $version
  channel = "beta"
  installer = $installer.Name
  installerSizeBytes = (Get-Item $copiedInstaller).Length
  sha256 = $hash
  signatureStatus = [string]$signature.Status
  sourceCommit = $gitSha
  buildSource = $env:AURA_BUILD_SOURCE
  buildLabel = $env:AURA_BUILD_LABEL
  builtAtUtc = [DateTime]::UtcNow.ToString("o")
}

$manifestPath = Join-Path $artifactDir "AURA-2-Beta-Build.json"
$manifest | ConvertTo-Json -Depth 4 | Set-Content $manifestPath -Encoding utf8

Write-Host ""
Write-Host "AURA-2 $version Windows Beta candidate built successfully."
Write-Host "Installer: $copiedInstaller"
Write-Host "SHA-256:   $hash"
Write-Host "Signature: $($signature.Status)"
Write-Host "Manifest:  $manifestPath"
Write-Host ""
Write-Host "Next gate: install this artifact on a clean/current-user Windows profile and complete the M009.3 smoke checklist."
