---
title: "Architecture Overview"
type: "architecture"
status: "approved"
created: "2026-02-03"
updated: "2026-02-03"
tags: ["tauri", "nextjs", "edge", "deepgram"]
---

# Architecture Overview - Voice2Text

## Summary
Voice2Text is a high-performance desktop transcription tool built on a hybrid architecture.

- **Frontend (Desktop)**: Tauri v2 (Rust + React) for minimal resource footprint.
- **Backend (API)**: Next.js 16 Edge Functions on Vercel for global low-latency.
- **Transcriber**: Deepgram Nova-2 via streaming/POST API.

## Core Flow
1. User triggers global hotkey (`Ctrl+Shift+V`).
2. Rust capture layer records audio from default input.
3. Audio buffer is sent as WAV/WebM to `/api/transcribe`.
4. Vercel Edge Function proxies request to Deepgram.
5. Resulting text is injected into the active window via `Enigo`.

## Decisions
- **Why Tauri?** Installer size < 10MB vs 80MB+ for Electron. Native performance for audio capture.
- **Why Edge Functions?** Cold starts < 50ms and global distribution close to the user.
