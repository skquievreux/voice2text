# 🎙️ Voice2Text

> **Instant, global voice transcription for Windows.**  
> Press a hotkey, speak, and get the text directly into your clipboard or active application. Powered by Vercel & optimized for minimal latency.

![Voice2Text App](https://via.placeholder.com/800x400?text=Voice2Text+App+Screenshot)

## ✨ Features

- **Global Hotkeys:** Press `F8` or `Ctrl+F12` anywhere in Windows to toggle recording.
- **Unlimited Recording:** Smart chunking allows recording for minutes or hours without limits.
- **Micro-Latency:** Uses Vercel Edge Functions & optimized 16-bit PCM compression for fast uploads.
- **Auto-Clipboard:** Transcribed text is automatically copied to your clipboard.
- **Crash-Proof:** Panic hooks and robust error handling ensuring stability.
- **Zero-Config:** Works out of the box with your default microphone.
- **Secure:** Standalone binary with built-in TLS security.

## 🚀 Installation

1. Go to the [Releases](https://github.com/skquievreux/voice2text/releases) page.
2. Download `Voice2Text_x.x.x_x64_en-US.exe` (NSIS Installer) or `.msi`.
3. Run the installer.
4. The app will launch and sit in your system tray.

## 🛠️ Development

### Prerequisites

- **Node.js**: v20+
- **Rust**: Latest Stable
- **PNPM**: v9+

### Setup

```bash
# Install dependencies
pnpm install

# Run Desktop App (Dev Mode)
pnpm tauri dev

# Run Web Backend (Dev Mode)
cd apps/web
pnpm dev
```

### Architecture

- **Frontend:** React + Vite (Typing effect, History list)
- **Backend (Desktop):** Rust (Tauri) for Audio Capture (cpal), compression (hound), and global shortcuts.
- **Backend (Cloud):** Next.js (Vercel) for API Proxy to Deepgram/OpenAI.

## 📦 Build

To create a production release locally:

```bash
pnpm tauri build
```

Artifacts will be in `apps/desktop/src-tauri/target/release/bundle/`.

## 🤖 CI/CD

This project uses **GitHub Actions** to automatically build and release the application.
- Pushing a tag `v*` triggers the Windows build pipeline.
- Dependabot keeps dependencies up to date.

## 📄 License

MIT © 2026 Quievreux
