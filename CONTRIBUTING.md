# Contributing to YT Downloader

Thank you for your interest in contributing! Here's how to get started.

## Getting Started

1. **Fork** the repository
2. **Clone** your fork:
   ```powershell
   git clone https://github.com/<your-username>/youtube-mp3-mp4.git
   cd youtube-mp3-mp4
   ```
3. **Install Rust** if you haven't already: [rustup.rs](https://rustup.rs/)
4. **Build and run** the app:
   ```powershell
   cargo run
   ```

## Development

- The Rust source is in `src/main.rs`
- The frontend HTML is in `assets/index.html` (embedded into the binary at compile time)
- The app icon is at `assets/icon.ico`, embedded via `build.rs`

After making changes, run `cargo run` to test locally.

## Submitting Changes

1. Create a feature branch:
   ```
   git checkout -b feature/my-change
   ```
2. Make your changes and commit with a clear message
3. Push to your fork and open a **Pull Request**

## Pull Request Guidelines

- Keep PRs focused — one feature or fix per PR
- Test your changes on Windows 10 or 11 before submitting
- Update the README if your change affects usage or setup

## Reporting Bugs

Open a [GitHub Issue](../../issues) with:

- Steps to reproduce
- Expected vs. actual behavior
- Windows version and any relevant error messages

## Security Issues

Please do **not** open a public issue for security vulnerabilities. See [SECURITY.md](SECURITY.md) for responsible disclosure instructions.

## Code of Conduct

By participating, you agree to follow our [Code of Conduct](CODE_OF_CONDUCT.md).
