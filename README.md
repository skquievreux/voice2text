# Voice2Text 🎙️

**Voice2Text** ist eine ultra-performante, minimalistische Desktop-Anwendung für Windows, die deine Sprache in Echtzeit in Text umwandelt und direkt in jede aktive Anwendung einfügt. 

Durch die Kombination von **Tauri 2.0 (Rust)** und **Next.js Edge Functions** bietet die App eine beispiellose Geschwindigkeit bei einem extrem geringen Ressourcenverbrauch (~8MB Installer).

## 🚀 Features
- **Globaler Hotkey**: Starte die Aufnahme jederzeit mit `Ctrl+Shift+V`.
- **Nahtlose Integration**: Der transkribierte Text wird direkt an der Cursor-Position eingefügt.
- **Vercel Edge API**: Minimale Latenz durch weltweit verteilte Edge-Server.
- **Deepgram Nova-2**: Industry-leading Sprache-zu-Text Genauigkeit (speziell für Deutsch optimiert).
- **Security First**: API-Keys sind sicher im Backend gekapselt; der Client nutzt JWT-Authentifizierung.
- **Business Ready**: Integriertes Lizenz-Management und Rate-Limiting.

## 🛠️ Tech Stack
- **Desktop**: [Tauri v2](https://tauri.app/) (Rust Backend, React/Vite Frontend)
- **Backend**: [Next.js 16](https://nextjs.org/) (Edge Runtime)
- **Infrastruktur**: [Vercel](https://vercel.com/) & [Vercel KV](https://vercel.com/storage/kv)
- **AI/STT**: [Deepgram Nova-2](https://www.deepgram.com/)
- **Monorepo**: [PNPM Workspaces](https://pnpm.io/) & [Turborepo](https://turbo.build/)

## 📦 Installation & Setup

### Voraussetzungen
- [Rust & Cargo](https://rustup.rs/) (Windows MSVC Toolchain)
- [Node.js](https://nodejs.org/) (>= 20.x)
- [PNPM](https://pnpm.io/)

### Lokale Entwicklung
1. **Repository klonen**
   ```bash
   git clone https://github.com/skquievreux/voice2text.git
   cd voice2text
   ```

2. **Abhängigkeiten installieren**
   ```bash
   pnpm install
   ```

3. **Umgebungsvariablen konfigurieren**
   - Kopiere `apps/web/.env.example` nach `apps/web/.env.local` und trage deinen `DEEPGRAM_API_KEY` sowie die Auth-Secrets ein.
   - Kopiere `apps/desktop/.env.example` nach `apps/desktop/.env`.

4. **Projekt starten**
   ```bash
   pnpm dev
   ```

## 📄 Lizenz
Dieses Projekt ist für Quievreux Consulting lizenziert.

## 🛡️ Governance
Dieses Projekt folgt dem **AI Agent Governance Framework v3.0**. Alle Releases werden automatisch über Semantic Release verwaltet.
