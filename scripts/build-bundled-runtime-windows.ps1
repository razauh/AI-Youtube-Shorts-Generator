param(
  [string]$Target = "windows-x86_64"
)

$ErrorActionPreference = "Stop"

if ($Target -ne "windows-x86_64") {
  throw "Unsupported target '$Target'. Expected windows-x86_64."
}

$RootDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$BuildDir = Join-Path $RootDir ".tmp-runtime-build\$Target"
$InputDir = Join-Path $RootDir "bundled-runtime-input\$Target"
$Requirements = @(
  (Join-Path $RootDir "requirements.txt"),
  (Join-Path $RootDir "requirements-local.txt")
)

foreach ($Requirement in $Requirements) {
  if (!(Test-Path $Requirement)) {
    throw "Missing requirements file: $Requirement"
  }
}

foreach ($Dir in @($BuildDir, $InputDir)) {
  if (Test-Path $Dir) {
    Remove-Item -Recurse -Force $Dir
  }
  New-Item -ItemType Directory -Force -Path $Dir | Out-Null
}

$ApiHeaders = @{
  "Accept" = "application/vnd.github+json"
  "User-Agent" = "ai-youtube-shorts-generator-runtime-builder"
}

$Release = Invoke-RestMethod `
  -Uri "https://api.github.com/repos/astral-sh/python-build-standalone/releases/latest" `
  -Headers $ApiHeaders

$PythonAsset = $Release.assets |
  Where-Object { $_.name -match '^cpython-3\.12\..*x86_64-pc-windows-msvc.*install_only.*\.tar\.gz$' } |
  Select-Object -First 1

if ($null -eq $PythonAsset) {
  throw "Could not find a python-build-standalone Windows x86_64 CPython 3.12 asset."
}

$PythonArchive = Join-Path $BuildDir $PythonAsset.name
Write-Host "[info] downloading $($PythonAsset.browser_download_url)"
Invoke-WebRequest -Uri $PythonAsset.browser_download_url -OutFile $PythonArchive

$ExtractDir = Join-Path $BuildDir "extract"
if (Test-Path $ExtractDir) {
  Remove-Item -Recurse -Force $ExtractDir
}
New-Item -ItemType Directory -Force -Path $ExtractDir | Out-Null
tar -xzf $PythonArchive -C $ExtractDir

$SourcePython = Join-Path $ExtractDir "python"
$RuntimePython = Join-Path $InputDir "python"
if (Test-Path $RuntimePython) {
  Remove-Item -Recurse -Force $RuntimePython
}
Copy-Item -Recurse $SourcePython $RuntimePython

$RootPython = Join-Path $InputDir "python.exe"
Copy-Item -Recurse -Force (Join-Path $RuntimePython "*") $InputDir

& $RootPython -m ensurepip --upgrade
& $RootPython -m pip install --upgrade pip
$SitePackages = Join-Path $InputDir "site-packages"
if (Test-Path $SitePackages) {
  Remove-Item -Recurse -Force $SitePackages
}
& $RootPython -m pip install --target $SitePackages -r $Requirements[0] -r $Requirements[1]
& $RootPython -m pip install --target $SitePackages imageio-ffmpeg

$LegacyDest = Join-Path $InputDir "python_legacy"
if (Test-Path $LegacyDest) {
  Remove-Item -Recurse -Force $LegacyDest
}
Copy-Item -Recurse (Join-Path $RootDir "python_legacy") $LegacyDest

$YtDlp = Join-Path $InputDir "yt-dlp.exe"
Write-Host "[info] downloading yt-dlp.exe"
Invoke-WebRequest `
  -Uri "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe" `
  -OutFile $YtDlp

$env:PYTHONPATH = (Resolve-Path $SitePackages).Path
$FfmpegSource = (& $RootPython -c "import imageio_ffmpeg; print(imageio_ffmpeg.get_ffmpeg_exe())").Trim()
Copy-Item -Force $FfmpegSource (Join-Path $InputDir "ffmpeg.exe")

$RuntimeInfo = @"
target=$Target
python_asset=$($PythonAsset.name)
yt_dlp_url=https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe
ffmpeg_source=$FfmpegSource
"@
$RuntimeInfo | Set-Content -Path (Join-Path $InputDir "RUNTIME_INFO.txt") -Encoding UTF8

& $RootPython -c "import requests, dotenv, yt_dlp, faster_whisper, openai, cv2, imageio_ffmpeg; print('runtime OK')"
& $YtDlp --version
& (Join-Path $InputDir "ffmpeg.exe") -version

Write-Host "[info] bundled runtime input prepared at $InputDir"
