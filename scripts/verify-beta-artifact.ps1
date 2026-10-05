$ErrorActionPreference = "Stop"

if ($env:OS -ne "Windows_NT") {
  throw "This verifier is intended for the AURA-2 Windows Beta artifact."
}

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$version = (Get-Content "package.json" -Raw | ConvertFrom-Json).version
$artifactDir = Join-Path $repoRoot "artifacts\beta\$version"
$manifestPath = Join-Path $artifactDir "AURA-2-Beta-Build.json"
$checksumPath = Join-Path $artifactDir "AURA-2-Windows-x64.sha256"

if (-not (Test-Path $manifestPath)) {
  throw "Beta build manifest not found: $manifestPath"
}
if (-not (Test-Path $checksumPath)) {
  throw "Beta checksum file not found: $checksumPath"
}

$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json

if ($manifest.schemaVersion -ne 2) {
  throw "Unsupported Beta build manifest schema: $($manifest.schemaVersion)"
}
if ($manifest.product -ne "AURA-2") {
  throw "Unexpected product in manifest: $($manifest.product)"
}
if ($manifest.version -ne $version) {
  throw "Manifest version $($manifest.version) does not match package version $version."
}
if ($manifest.channel -ne "beta") {
  throw "Unexpected release channel in manifest: $($manifest.channel)"
}
if (-not $manifest.sourceCommit -or $manifest.sourceCommit -eq "unknown") {
  throw "Build manifest does not contain a traceable source commit."
}
if (-not $manifest.buildSource) {
  throw "Build manifest does not identify the build source."
}
if (-not $manifest.buildLabel) {
  throw "Build manifest does not identify the build label."
}
if (-not $manifest.installer) {
  throw "Manifest does not identify an installer."
}

$installerPath = Join-Path $artifactDir $manifest.installer
if (-not (Test-Path $installerPath)) {
  throw "Installer declared by manifest does not exist: $installerPath"
}

$installer = Get-Item $installerPath
if ($installer.Length -le 0) {
  throw "Installer is empty."
}
if ([int64]$manifest.installerSizeBytes -ne $installer.Length) {
  throw "Installer size no longer matches the build manifest."
}

$actualHash = (Get-FileHash $installerPath -Algorithm SHA256).Hash.ToLower()
$manifestHash = ([string]$manifest.sha256).ToLower()
if ($actualHash -ne $manifestHash) {
  throw "Installer SHA-256 does not match the build manifest."
}

$checksumLine = (Get-Content $checksumPath -Raw).Trim()
$expectedChecksumLine = "$manifestHash  $($manifest.installer)"
if ($checksumLine -ne $expectedChecksumLine) {
  throw "Checksum file does not match the build manifest."
}

$signature = Get-AuthenticodeSignature $installerPath
if ([string]$signature.Status -ne [string]$manifest.signatureStatus) {
  throw "Authenticode status changed since the build manifest was created."
}

Write-Host ""
Write-Host "AURA-2 Beta artifact verified."
Write-Host "Version:    $version"
Write-Host "Installer:  $($manifest.installer)"
Write-Host "Bytes:      $($installer.Length)"
Write-Host "SHA-256:    $actualHash"
Write-Host "Signature:  $($signature.Status)"
Write-Host "Commit:     $($manifest.sourceCommit)"
Write-Host "Build:      $($manifest.buildLabel) · $($manifest.buildSource)"
Write-Host "Built UTC:  $($manifest.builtAtUtc)"
