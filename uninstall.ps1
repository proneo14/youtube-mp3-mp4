# uninstall.ps1 — Remove YT Downloader and bundled tools

# --- Require admin for Program Files access ---
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    $scriptPath = $MyInvocation.MyCommand.Definition
    Start-Process powershell.exe "-NoProfile -ExecutionPolicy Bypass -File `"$scriptPath`"" -Verb RunAs -Wait
    exit
}

# Remove from Program Files
$installDir = "$env:ProgramFiles\YTDownloader"
$shortcutPath = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs\YT Downloader.lnk"
if (Test-Path $shortcutPath) { Remove-Item $shortcutPath -Force }
if (Test-Path $installDir)   { Remove-Item $installDir -Recurse -Force }

# Also clean old LocalAppData install if it exists
$oldDir = "$env:LOCALAPPDATA\YTDownloader"
$oldShortcut = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\YT Downloader.lnk"
if (Test-Path $oldShortcut) { Remove-Item $oldShortcut -Force -ErrorAction SilentlyContinue }
if (Test-Path $oldDir)      { Remove-Item $oldDir -Recurse -Force -ErrorAction SilentlyContinue }

Write-Host "YT Downloader uninstalled (app, yt-dlp, ffmpeg all removed)." -ForegroundColor Green
