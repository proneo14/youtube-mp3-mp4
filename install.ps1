# install.ps1 — Build release binary, bundle yt-dlp + ffmpeg, install to Program Files
# Run from the project root: .\install.ps1

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$projectDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
Set-Location $projectDir

# --- Build first (no admin needed) ---
Write-Host "`n=== Building YT Downloader (release) ===" -ForegroundColor Green
cargo build --release
if ($LASTEXITCODE -ne 0) { Write-Host "Build failed." -ForegroundColor Red; exit 1 }

$exeSrc = "$projectDir\target\release\yt-downloader.exe"

# --- Run the install portion as admin ---
$installScript = @"
`$ErrorActionPreference = 'Stop'
`$installDir   = "`$env:ProgramFiles\YTDownloader"
`$binDir       = "`$installDir\bin"
`$exeDest      = "`$installDir\yt-downloader.exe"
`$ytdlpPath    = "`$binDir\yt-dlp.exe"
`$ffmpegDir    = "`$binDir\ffmpeg"
`$ffmpegZip    = "`$binDir\ffmpeg.zip"

Write-Host "`n=== Installing to `$installDir ===" -ForegroundColor Green
New-Item -ItemType Directory -Path `$binDir -Force | Out-Null

# Clean old installs
`$oldDir = "`$env:LOCALAPPDATA\YTDownloader"
`$oldShortcut = "`$env:APPDATA\Microsoft\Windows\Start Menu\Programs\YT Downloader.lnk"
if (Test-Path `$oldShortcut) { Remove-Item `$oldShortcut -Force -ErrorAction SilentlyContinue }
if (Test-Path `$oldDir)      { Remove-Item `$oldDir -Recurse -Force -ErrorAction SilentlyContinue }

# Copy exe
Copy-Item -Path '$exeSrc' -Destination `$exeDest -Force

# Download yt-dlp
if (-not (Test-Path `$ytdlpPath)) {
    Write-Host "`n=== Downloading yt-dlp ===" -ForegroundColor Cyan
    Invoke-WebRequest -Uri 'https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe' -OutFile `$ytdlpPath -UseBasicParsing
    Write-Host "yt-dlp downloaded."
} else {
    Write-Host "yt-dlp already present, skipping download."
}

# Download ffmpeg
`$ffmpegExe = Get-ChildItem -Path `$ffmpegDir -Filter 'ffmpeg.exe' -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not `$ffmpegExe) {
    Write-Host "`n=== Downloading ffmpeg ===" -ForegroundColor Cyan
    Invoke-WebRequest -Uri 'https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip' -OutFile `$ffmpegZip -UseBasicParsing
    Expand-Archive -Path `$ffmpegZip -DestinationPath `$ffmpegDir -Force
    Remove-Item `$ffmpegZip -Force
    Write-Host "ffmpeg downloaded and extracted."
} else {
    Write-Host "ffmpeg already present, skipping download."
}

# Create Start Menu shortcut (system-wide)
`$startMenu = "`$env:ProgramData\Microsoft\Windows\Start Menu\Programs"
`$shortcutPath = "`$startMenu\YT Downloader.lnk"
`$shell = New-Object -ComObject WScript.Shell
`$shortcut = `$shell.CreateShortcut(`$shortcutPath)
`$shortcut.TargetPath = `$exeDest
`$shortcut.WorkingDirectory = `$installDir
`$shortcut.Description = 'YouTube to MP3/MP4 Downloader'
`$shortcut.IconLocation = "`$exeDest,0"
`$shortcut.Save()

Write-Host "`n=== Done! ===" -ForegroundColor Green
Write-Host "Installed to: `$exeDest"
Write-Host "yt-dlp:  `$ytdlpPath"
Write-Host "ffmpeg:  `$ffmpegDir"
Write-Host "Shortcut: `$shortcutPath"
Write-Host "`nYou can now find 'YT Downloader' in Windows Search." -ForegroundColor Cyan
Write-Host "Press any key to close..."; `$null = `$Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')
"@

$tempScript = "$env:TEMP\yt-dl-install.ps1"
$installScript | Out-File -FilePath $tempScript -Encoding UTF8 -Force

Write-Host "`nElevating to install..." -ForegroundColor Yellow
Start-Process powershell.exe "-NoProfile -ExecutionPolicy Bypass -File `"$tempScript`"" -Verb RunAs -Wait
Remove-Item $tempScript -Force -ErrorAction SilentlyContinue

Write-Host "`nInstall complete!" -ForegroundColor Green
