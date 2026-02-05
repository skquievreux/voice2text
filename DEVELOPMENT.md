# Development Guide

## Quick Start

### Web / Backend
```bash
cd c:\CODE\GIT\Voice2Text-Web
pnpm dev
```
> Runs the Next.js backend at http://localhost:3000

### Desktop App
From the monorepo root (`c:\CODE\GIT\Voice2Text`):

```bash
# Start the desktop app in dev mode (hot reload)
pnpm dev:desktop
```

### Full Build (Release)
```bash
pnpm build
```

## Troubleshooting
- If "Registering..." hangs -> Check if Web Backend is running.
- If Tray icon is weird -> Rebuild app (we fixed the logic in `lib.rs`).
