param(
  [string]$ArtifactDir = "$(Join-Path (Get-Location) 'app/src-tauri/target/release/bundle')"
)

$ErrorActionPreference = 'Stop'
$RootDir = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$LogDir = Join-Path $RootDir '.logs'
$Timestamp = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ')
$LogFile = Join-Path $LogDir "installer-smoke-windows-$Timestamp.log"

New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
Start-Transcript -Path $LogFile -Append | Out-Null

try {
  Write-Host "[info] Windows installer smoke started: $Timestamp"
  Write-Host "[info] artifact dir: $ArtifactDir"

  if (-not (Test-Path -Path $ArtifactDir -PathType Container)) {
    throw "artifact directory does not exist"
  }

  $artifacts = @(
    Get-ChildItem -Path $ArtifactDir -Recurse -Include '*.msi','*.exe','*.nsis.zip' -File
  )

  if ($artifacts.Count -eq 0) {
    throw "no Windows installer artifacts found"
  }

  foreach ($artifact in $artifacts) {
    if ($artifact.Length -le 0) {
      throw "empty artifact: $($artifact.FullName)"
    }
    Write-Host "[pass] found artifact: $($artifact.FullName)"
  }

  $configPath = Join-Path $RootDir 'app/src-tauri/tauri.conf.json'
  $indexPath = Join-Path $RootDir 'app/index.html'
  $configText = Get-Content -Raw -Path $configPath
  $indexText = Get-Content -Raw -Path $indexPath
  if (($configText + $indexText) -match 'YOUR_CLOUDFLARE_SUBDOMAIN|REPLACE_WITH_TAURI_UPDATER_PUBLIC_KEY|AI Shorts App|Signal Forge') {
    throw "release config still contains placeholder updater or stale product text"
  }

  Write-Host "[pass] Windows installer smoke checks completed"
  Write-Host "[info] log written to $LogFile"
} finally {
  Stop-Transcript | Out-Null
}
