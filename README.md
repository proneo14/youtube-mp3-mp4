# YT Downloader

A lightweight Windows desktop app to download YouTube videos as MP3 or MP4 files.

![Rust](https://img.shields.io/badge/Rust-000?logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows%2010%2F11-0078D6?logo=windows&logoColor=white)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

## Features

- **Auto-preview** — Paste a YouTube URL and instantly see the video title, thumbnail, channel, and duration
- **MP3 or MP4** — Pick your format; switching re-triggers the preview
- **Native Windows app** — Runs as a standalone `.exe` with a WebView2 UI (no browser needed)
- **Bundled dependencies** — yt-dlp and ffmpeg are downloaded automatically during install
- **Start Menu integration** — Searchable via Windows Search after install

## Requirements

- **Windows 10 (1803+) or Windows 11**
- **Rust toolchain** — only needed to build from source ([install](https://rustup.rs/))

## Quick Start

```powershell
git clone https://github.com/proneo14/youtube-mp3-mp4.git
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
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── SECURITY.md
└── README.md
```

## How It Works

The app runs a local HTTP server on a random port and opens a native window (WebView2) pointed at it. The server has two endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/preview` | POST | Calls `yt-dlp --dump-json` to fetch video metadata without downloading |
| `/download` | POST | Runs yt-dlp to download and convert the video, then streams the file back |

yt-dlp and ffmpeg binaries are bundled in `bin/` next to the exe. The app finds them automatically — no PATH configuration needed.

## Contributing

Contributions are welcome! Please read the [Contributing Guide](CONTRIBUTING.md) before submitting a pull request.

## Security

To report a vulnerability, please see [SECURITY.md](SECURITY.md).

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).

## Disclaimer

> **This project is strictly for educational purposes only.** Downloading copyrighted content from YouTube or any other platform without the permission of the content owner is **illegal** and a violation of YouTube's Terms of Service. The authors of this project do not condone or encourage the unauthorized downloading of copyrighted material. **Use this tool at your own risk** — you are solely responsible for ensuring your use complies with all applicable laws and regulations.

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.
