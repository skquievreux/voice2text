# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-02-03

### Added
- **Backend Analytics:** Integrated Supabase (Table: `v2t_transcriptions`) for tracking transcription metadata (duration, words, cost).
- **Audio Storage:** Integrated Cloudflare R2 for persistent storage of recorded audio.
- **Admin Dashboard:** New web interface for monitoring usage, costs, and system status.
- **Audio Chunking:** Support for unlimited recording duration by splitting audio into <3MB chunks.
- **Compression:** Implemented 16-bit PCM (Int16) audio format to reduce payload size by 50%.
- **Hotkeys:** Added `F8` as an alternative global shortcut alongside `Ctrl+F12`.
- **UI:** Added transcription history list and auto-clipboard functionality.
- **CI/CD:** Automated Windows build pipeline via GitHub Actions.
- **Packaging:** Standalone binary support using `rustls` (no external OpenSSL dependency).
- **Installer:** NSIS configuration for professional setup experience.

### Fixed
- Fixed `413 Payload Too Large` error for recordings > 1 minute.
- Fixed `cpal` audio stream crashes by checking device capabilities dynamically.
- Fixed compilation warnings and unused variables in Rust backend.
