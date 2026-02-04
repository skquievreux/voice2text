# 🎙️ Voice2Text Monorepo

> **Hochperformante, globale Sprachtranskription für Windows.**  
> Per Hotkey aufnehmen, sprechen und den Text sofort in der aktiven Anwendung oder Zwischenablage erhalten.

Dieses Monorepo verwaltet das gesamte Voice2Text-Ökosystem, bestehend aus einem Desktop-Client (Tauri) und einem performanten Backend (Next.js).

## ✨ Features
- **Globale Hotkeys:** `F8` oder `Ctrl+F12` zur Steuerung der Aufnahme in jeder Windows-App.
- **Unbegrenzte Aufnahme:** Automatisches Datei-Splitting (<3MB Chunks) für stundenlange Aufnahmen.
- **Edge-Powered:** Nutzung von Vercel Edge Functions für globale Verfügbarkeit und minimale Latenz.
- **Privat & Sicher:** Lokale Vorverarbeitung (16-bit PCM) und verschlüsselte Übertragung.
- **Automatisierung:** Integrierung in den Workflow durch automatische Clipboard-Injektion.

## 📂 Repository Struktur
Das Projekt ist als PNPM-Workspace organisiert:
- `apps/desktop`: Der Windows Client (Tauri + React + Rust).
- `apps/web`: Das API-Backend (Next.js), optimiert für Vercel.
- `docs/`: Umfassende Dokumentation nach Governance v3.0 Standard.

## 🛠️ Entwicklung

### Voraussetzungen
- **Node.js**: v20+
- **Rust**: Aktuelle Stable-Version (für den Desktop-Client)
- **PNPM**: v10.11.0+

### Setup
```bash
# Abhängigkeiten installieren
pnpm install

# Desktop App (Entwicklungsmodus)
pnpm run dev --filter desktop

# Web Backend (Entwicklungsmodus)
pnpm run dev --filter web
```

## 🏗️ Build & Release
Der Build-Prozess ist via GitHub Actions automatisiert. Manuelle Builds können wie folgt erstellt werden:
```bash
# Lokales Desktop-Bundle erstellen
pnpm run build --filter desktop
```

## 📚 Weiterführende Dokumentation
- [Architektur-ADRs](./docs/01-architecture/)
- [Implementierungs-Guides](./docs/02-implementation/)
- [Business-Anforderungen](./docs/04-business/)

## ⚖️ Governance
Dieses Projekt folgt dem **AI Agent Governance Framework v3.0**. Alle Releases werden rein über Semantic-Rollouts gesteuert.

---
**Lizenz:** MIT © 2026 Quievreux  
**Version:** 1.0.0 (Release via CI/CD)
