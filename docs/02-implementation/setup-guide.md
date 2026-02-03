---
title: "Development Setup"
type: "implementation"
status: "approved"
created: "2026-02-03"
updated: "2026-02-03"
tags: ["setup", "pnpm", "rust"]
---

# Development Setup

## Prerequisites
- **Node.js**: >= 20.0.0
- **PNPM**: >= 10.0.0
- **Rust/Cargo**: Required for Tauri builds.
- **C++ Build Tools**: Required on Windows.

## Installation
1. Enable Corepack: `corepack enable`
2. Install dependencies: `pnpm install`
3. Set up environment variables:
   - Copy `apps/web/.env.example` -> `apps/web/.env.local`
   - Copy `apps/desktop/.env.example` -> `apps/desktop/.env`

## Running Locally
- **Full Stack**: `pnpm dev` (Runs Next.js and Tauri in parallel)
- **Web only**: `pnpm --filter web dev`
- **Desktop only**: `pnpm --filter desktop tauri dev`
