# YT Downloader

A lightweight Windows desktop app to download YouTube videos as MP3 or MP4 files.

![Rust](https://img.shields.io/badge/Rust-000?logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows%2010%2F11-0078D6?logo=windows&logoColor=white)

## Features

- **Auto-preview** — Paste a YouTube URL and instantly see the video title, thumbnail, channel, and duration
- **MP3 or MP4** — Pick your format; switching re-triggers the preview
- **Native Windows app** — Runs as a standalone `.exe` with a WebView2 UI (no browser needed)
- **Bundled dependencies** — yt-dlp and ffmpeg are downloaded automatically during install
- **Start Menu integration** — Searchable via Windows Search after install

## Requirements

- **Windows 10 (1803+) or Windows 11**
- **Rust toolchain** (only needed to build from source)

## Install

```powershell
git clone <repo-url>
cd youtube-mp3-mp4
.\install.ps1
```

This will:
1. Build the release binary with `cargo build --release`
2. Prompt for admin access to install to `C:\Program Files\YTDownloader\`
3. Download **yt-dlp** and **ffmpeg** into `Program Files\YTDownloader\bin\`
4. Create a Start Menu shortcut (visible in Windows Search)

## Uninstall

```powershell
.\uninstall.ps1
```

Removes the app, bundled yt-dlp/ffmpeg, and the Start Menu shortcut.

## Usage

1. Open **YT Downloader** from the Start Menu or Windows Search
2. Paste a YouTube URL — the preview loads automatically
3. Select **MP3** or **MP4**
4. Click **Download**

Downloaded files are saved to your browser's default download location.

## Project Structure

```
├── src/
│   └── main.rs          # Rust app: webview window + HTTP server + yt-dlp integration
├── assets/
│   ├── index.html       # Frontend UI (embedded into the binary)
│   └── icon.ico         # App icon
├── build.rs             # Embeds the icon into the Windows exe
├── install.ps1          # Build + install + download dependencies
├── uninstall.ps1        # Clean removal
├── Cargo.toml           # Rust dependencies
└── Cargo.lock
```

## How It Works

The app runs a local HTTP server on a random port and opens a native window (WebView2) pointed at it. The server has two endpoints:

- `POST /preview` — Calls `yt-dlp --dump-json` to fetch video metadata without downloading
- `POST /download` — Runs yt-dlp to download and convert the video, then streams the file back

yt-dlp and ffmpeg binaries are bundled in `bin/` next to the exe. The app finds them automatically — no PATH configuration needed.

## License

MIT
