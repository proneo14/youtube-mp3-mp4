# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| latest  | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in YT Downloader, please report it responsibly.

Include the following in your report:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response Timeline

- **Acknowledgment** — within 48 hours
- **Initial assessment** — within 5 business days
- **Fix and disclosure** — coordinated with the reporter

## Scope

This policy covers:

- The YT Downloader desktop application (`yt-downloader.exe`)
- The install/uninstall scripts (`install.ps1`, `uninstall.ps1`)
- The embedded web frontend (`assets/index.html`)

Out of scope:

- Vulnerabilities in third-party dependencies (yt-dlp, ffmpeg) — please report those to their respective projects
- Issues requiring physical access to the machine

## Security Design

- The app binds its HTTP server to `127.0.0.1` (localhost only) — it is not accessible from the network
- A random port is used on each launch to avoid port conflicts
- Temporary download files are cleaned up after each conversion
- No user data is collected, stored, or transmitted
