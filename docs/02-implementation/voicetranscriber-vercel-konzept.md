# VoiceTranscriber - Vercel Backend Implementation Guide

**Version:** 1.0  
**Datum:** 03.02.2026  
**Autor:** Don Key - Quievreux Consulting

---

## 🎯 Executive Summary

Dieses Dokument beschreibt die vollständige Implementierung von VoiceTranscriber als System-Tray-Tool mit Vercel Backend für Speech-to-Text. Die Architektur nutzt:

- **Desktop Client:** Tauri (Rust + TypeScript) - 8 MB Installer
- **Backend:** Vercel Edge Functions (kostenlos bis 2k Users)
- **STT Provider:** Deepgram Nova-2 oder OpenAI GPT-4o Mini
- **Authentifizierung:** JWT + Vercel KV
- **Rate Limiting:** Upstash Redis via Vercel KV

**Geschätzte Entwicklungszeit:** 3-4 Wochen  
**Kosten bis 2k Users:** €0 Vercel + ~$100/Monat STT API

---

## 📐 Systemarchitektur

```
┌──────────────────────────────────────────────────────────┐
│                    Desktop Client                         │
│                  (Tauri 2.0 + React)                     │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  System Tray Icon                               │    │
│  │  └─ Global Hotkey Listener (Ctrl+Shift+V)      │    │
│  │  └─ Audio Capture (PortAudio/CPAL)             │    │
│  │  └─ Text Injection (Enigo)                     │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Auth Module                                     │    │
│  │  └─ JWT Token Storage (Secure Keychain)        │    │
│  │  └─ License Key Validation                     │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  API Client                                      │    │
│  │  └─ HTTPS Client (reqwest)                     │    │
│  │  └─ Retry Logic + Error Handling               │    │
│  │  └─ Audio Compression (Opus/AAC)               │    │
│  └─────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────┘
                            │
                            │ HTTPS (TLS 1.3)
                            │
                            ▼
┌──────────────────────────────────────────────────────────┐
│              Vercel Edge Functions                        │
│           (Global CDN - 270+ Locations)                  │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  /api/auth/register                             │    │
│  │  └─ License Key Generation                     │    │
│  │  └─ User Creation in KV Store                  │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  /api/auth/login                                │    │
│  │  └─ JWT Token Generation (7 Days)              │    │
│  │  └─ Refresh Token (30 Days)                    │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  /api/transcribe                                │    │
│  │  └─ JWT Validation                             │    │
│  │  └─ Rate Limiting (Tier-based)                 │    │
│  │  └─ Usage Tracking                             │    │
│  │  └─ Proxy to STT Provider                      │    │
│  └─────────────────────────────────────────────────┘    │
│                                                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │  /api/usage                                     │    │
│  │  └─ Get User's Monthly Usage                   │    │
│  └─────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────┘
                            │
                            ▼
            ┌───────────────────────────────┐
            │     Vercel KV (Redis)         │
            ├───────────────────────────────┤
            │  - User Data                  │
            │  - JWT Refresh Tokens         │
            │  - Rate Limit Counters        │
            │  - Usage Statistics           │
            └───────────────────────────────┘
                            │
                            ▼
            ┌───────────────────────────────┐
            │   STT Provider (Deepgram)     │
            ├───────────────────────────────┤
            │  POST /v1/listen              │
            │  - Audio → Text               │
            │  - Language Detection         │
            │  - Punctuation                │
            └───────────────────────────────┘
```

---

## 🔐 API Key Management & Security

### **1. Environment Variables (Vercel)**

```bash
# .env.local (für lokale Entwicklung)
# Vercel Dashboard für Production

# STT Provider
DEEPGRAM_API_KEY=your_deepgram_key_here

# JWT Secrets
JWT_SECRET=your-super-secret-jwt-key-minimum-32-chars
JWT_REFRESH_SECRET=your-refresh-token-secret-key

# Vercel KV (automatisch injected von Vercel)
KV_URL=
KV_REST_API_URL=
KV_REST_API_TOKEN=
KV_REST_API_READ_ONLY_TOKEN=

# License Key Encryption
LICENSE_ENCRYPTION_KEY=your-license-encryption-key-32-chars

# Optional: Stripe für Payments
STRIPE_SECRET_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
```

### **2. Vercel Setup (Step-by-Step)**

```bash
# 1. Projekt erstellen
mkdir voicetranscriber-backend
cd voicetranscriber-backend
pnpm create next-app@latest . --typescript --tailwind --app --no-src-dir

# 2. Vercel CLI installieren
pnpm add -g vercel

# 3. Vercel Projekt verknüpfen
vercel login
vercel link

# 4. KV Database erstellen (im Vercel Dashboard)
# Storage → KV → Create Database → voicetranscriber-kv

# 5. Environment Variables setzen
vercel env add DEEPGRAM_API_KEY
vercel env add JWT_SECRET
vercel env add JWT_REFRESH_SECRET
vercel env add LICENSE_ENCRYPTION_KEY

# 6. Deployen
vercel --prod
```

### **3. API Key Sicherheit - Best Practices**

```typescript
// ❌ NIEMALS im Client-Code:
const DEEPGRAM_KEY = 'abcd1234...' // EXPOSED!

// ✅ Immer auf Server-Seite:
// app/api/transcribe/route.ts
const DEEPGRAM_KEY = process.env.DEEPGRAM_API_KEY

// ✅ Client erhält nur JWT Token:
// Desktop Client speichert nur:
{
  accessToken: 'jwt.token.here',  // 7 Tage gültig
  refreshToken: 'refresh.token',   // 30 Tage gültig
  userId: 'user_abc123'
}
```

### **4. License Key System**

```typescript
// lib/license.ts
import crypto from 'crypto'

export class LicenseManager {
  private encryptionKey: Buffer
  
  constructor() {
    const key = process.env.LICENSE_ENCRYPTION_KEY!
    this.encryptionKey = Buffer.from(key, 'hex')
  }
  
  // Format: VT-XXXX-XXXX-XXXX-XXXX
  generateLicenseKey(tier: 'free' | 'basic' | 'pro'): string {
    const data = {
      tier,
      created: Date.now(),
      random: crypto.randomBytes(8).toString('hex')
    }
    
    const json = JSON.stringify(data)
    const cipher = crypto.createCipheriv('aes-256-gcm', this.encryptionKey, iv)
    const encrypted = Buffer.concat([cipher.update(json, 'utf8'), cipher.final()])
    
    // Format als XXXX-XXXX-XXXX-XXXX
    const key = encrypted.toString('hex').toUpperCase()
    return `VT-${key.slice(0,4)}-${key.slice(4,8)}-${key.slice(8,12)}-${key.slice(12,16)}`
  }
  
  validateLicenseKey(key: string): { valid: boolean; tier?: string } {
    try {
      // Entferne VT- Prefix und Dashes
      const hex = key.replace('VT-', '').replace(/-/g, '')
      const encrypted = Buffer.from(hex, 'hex')
      
      const decipher = crypto.createDecipheriv('aes-256-gcm', this.encryptionKey, iv)
      const decrypted = Buffer.concat([decipher.update(encrypted), decipher.final()])
      const data = JSON.parse(decrypted.toString('utf8'))
      
      return { valid: true, tier: data.tier }
    } catch {
      return { valid: false }
    }
  }
}
```

---

## 🚀 Vercel Backend Implementation

### **1. Projekt-Struktur**

```
voicetranscriber-backend/
├── app/
│   ├── api/
│   │   ├── auth/
│   │   │   ├── login/
│   │   │   │   └── route.ts
│   │   │   ├── register/
│   │   │   │   └── route.ts
│   │   │   └── refresh/
│   │   │       └── route.ts
│   │   ├── transcribe/
│   │   │   └── route.ts
│   │   ├── usage/
│   │   │   └── route.ts
│   │   └── webhook/
│   │       └── stripe/
│   │           └── route.ts
│   ├── layout.tsx
│   └── page.tsx (Marketing Landing Page)
├── lib/
│   ├── auth.ts          # JWT Utils
│   ├── db.ts            # Vercel KV Wrapper
│   ├── license.ts       # License Key Management
│   ├── rate-limit.ts    # Rate Limiting
│   └── stt.ts           # STT Provider Clients
├── types/
│   └── index.ts         # TypeScript Types
├── middleware.ts        # Request Logging
├── package.json
├── tsconfig.json
└── vercel.json
```

### **2. Core Libraries**

```json
// package.json
{
  "dependencies": {
    "next": "14.2.0",
    "react": "^18.3.0",
    "@vercel/kv": "^2.0.0",
    "@upstash/ratelimit": "^2.0.0",
    "jose": "^5.2.0",              // JWT (Edge-kompatibel)
    "zod": "^3.22.0"               // Schema Validation
  },
  "devDependencies": {
    "@types/node": "^20",
    "typescript": "^5"
  }
}
```

### **3. Authentication API**

```typescript
// app/api/auth/register/route.ts
import { NextRequest } from 'next/server'
import { z } from 'zod'
import { kv } from '@vercel/kv'
import { LicenseManager } from '@/lib/license'

export const runtime = 'edge'

const registerSchema = z.object({
  email: z.string().email(),
  licenseKey: z.string().regex(/^VT-[A-F0-9]{4}-[A-F0-9]{4}-[A-F0-9]{4}-[A-F0-9]{4}$/),
  deviceId: z.string().uuid()
})

export async function POST(req: NextRequest) {
  try {
    const body = await req.json()
    const { email, licenseKey, deviceId } = registerSchema.parse(body)
    
    // 1. Validate License Key
    const licenseManager = new LicenseManager()
    const { valid, tier } = licenseManager.validateLicenseKey(licenseKey)
    
    if (!valid) {
      return Response.json(
        { error: 'Invalid license key' },
        { status: 400 }
      )
    }
    
    // 2. Check if already used
    const existingLicense = await kv.get(`license:${licenseKey}`)
    if (existingLicense) {
      return Response.json(
        { error: 'License key already activated' },
        { status: 400 }
      )
    }
    
    // 3. Create User
    const userId = crypto.randomUUID()
    const user = {
      id: userId,
      email,
      tier,
      licenseKey,
      devices: [deviceId],
      createdAt: Date.now(),
      usage: {
        minutesThisMonth: 0,
        requestsToday: 0
      }
    }
    
    await kv.set(`user:${userId}`, user)
    await kv.set(`license:${licenseKey}`, userId)
    await kv.set(`email:${email}`, userId)
    
    // 4. Generate JWT Tokens
    const { accessToken, refreshToken } = await generateTokens(userId, tier)
    
    return Response.json({
      accessToken,
      refreshToken,
      user: {
        id: userId,
        email,
        tier
      }
    })
    
  } catch (error) {
    if (error instanceof z.ZodError) {
      return Response.json(
        { error: 'Invalid input', details: error.errors },
        { status: 400 }
      )
    }
    
    console.error('Registration error:', error)
    return Response.json(
      { error: 'Internal server error' },
      { status: 500 }
    )
  }
}
```

### **4. Transcription API (Core)**

```typescript
// app/api/transcribe/route.ts
import { NextRequest } from 'next/server'
import { verifyAccessToken } from '@/lib/auth'
import { getRateLimiter } from '@/lib/rate-limit'
import { DeepgramClient } from '@/lib/stt'
import { kv } from '@vercel/kv'

export const runtime = 'edge'
export const maxDuration = 30 // seconds

export async function POST(req: NextRequest) {
  try {
    // 1. Verify JWT Token
    const token = req.headers.get('authorization')?.replace('Bearer ', '')
    if (!token) {
      return Response.json({ error: 'No token provided' }, { status: 401 })
    }
    
    const payload = await verifyAccessToken(token)
    if (!payload) {
      return Response.json({ error: 'Invalid token' }, { status: 401 })
    }
    
    const userId = payload.sub as string
    const tier = payload.tier as string
    
    // 2. Rate Limiting
    const rateLimiter = getRateLimiter(tier)
    const { success, remaining, reset } = await rateLimiter.limit(userId)
    
    if (!success) {
      return Response.json(
        { 
          error: 'Rate limit exceeded',
          remaining: 0,
          resetAt: reset
        },
        { 
          status: 429,
          headers: {
            'X-RateLimit-Remaining': '0',
            'X-RateLimit-Reset': reset.toString()
          }
        }
      )
    }
    
    // 3. Check Monthly Quota (for Free tier)
    if (tier === 'free') {
      const usage = await kv.get<number>(`usage:${userId}:${getCurrentMonth()}`)
      if (usage && usage >= 300) { // 300 minutes limit
        return Response.json(
          { error: 'Monthly quota exceeded. Upgrade to Basic for unlimited.' },
          { status: 402 } // Payment Required
        )
      }
    }
    
    // 4. Get Audio File
    const formData = await req.formData()
    const audioFile = formData.get('audio') as File
    
    if (!audioFile) {
      return Response.json({ error: 'No audio file provided' }, { status: 400 })
    }
    
    // 5. Transcribe via Deepgram
    const client = new DeepgramClient()
    const result = await client.transcribe(
      await audioFile.arrayBuffer(),
      {
        language: 'de',
        model: 'nova-2',
        punctuate: true,
        smart_format: true
      }
    )
    
    // 6. Track Usage
    const durationMinutes = result.metadata.duration / 60
    await kv.hincrby(
      `usage:${userId}:${getCurrentMonth()}`,
      'minutes',
      durationMinutes
    )
    
    await kv.lpush(
      `history:${userId}`,
      {
        timestamp: Date.now(),
        duration: result.metadata.duration,
        text: result.transcript.substring(0, 100), // First 100 chars
        language: result.metadata.language
      }
    )
    
    // 7. Return Result
    return Response.json({
      text: result.transcript,
      confidence: result.confidence,
      duration: result.metadata.duration,
      usage: {
        remaining,
        resetAt: reset
      }
    }, {
      headers: {
        'X-RateLimit-Remaining': remaining.toString(),
        'X-RateLimit-Reset': reset.toString()
      }
    })
    
  } catch (error) {
    console.error('Transcription error:', error)
    
    // Retry-able errors
    if (error instanceof Error && error.message.includes('timeout')) {
      return Response.json(
        { error: 'Request timeout. Please try again with shorter audio.' },
        { status: 504 }
      )
    }
    
    return Response.json(
      { error: 'Transcription failed. Please try again.' },
      { status: 500 }
    )
  }
}

function getCurrentMonth(): string {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`
}
```

### **5. Rate Limiting Implementation**

```typescript
// lib/rate-limit.ts
import { Ratelimit } from '@upstash/ratelimit'
import { kv } from '@vercel/kv'

const RATE_LIMITS = {
  free: {
    requests: 10,      // 10 requests
    window: '1 h'      // per hour
  },
  basic: {
    requests: 100,
    window: '1 h'
  },
  pro: {
    requests: 1000,
    window: '1 h'
  }
}

export function getRateLimiter(tier: string) {
  const config = RATE_LIMITS[tier as keyof typeof RATE_LIMITS] || RATE_LIMITS.free
  
  return new Ratelimit({
    redis: kv,
    limiter: Ratelimit.slidingWindow(config.requests, config.window),
    prefix: `ratelimit:${tier}`,
    analytics: true, // Optional: Track in Vercel Dashboard
  })
}
```

### **6. STT Provider Client**

```typescript
// lib/stt.ts
export class DeepgramClient {
  private apiKey: string
  private baseUrl = 'https://api.deepgram.com/v1'
  
  constructor() {
    this.apiKey = process.env.DEEPGRAM_API_KEY!
  }
  
  async transcribe(
    audioBuffer: ArrayBuffer,
    options: {
      language: string
      model: string
      punctuate: boolean
      smart_format: boolean
    }
  ): Promise<TranscriptionResult> {
    const queryParams = new URLSearchParams({
      language: options.language,
      model: options.model,
      punctuate: options.punctuate.toString(),
      smart_format: options.smart_format.toString()
    })
    
    const response = await fetch(
      `${this.baseUrl}/listen?${queryParams}`,
      {
        method: 'POST',
        headers: {
          'Authorization': `Token ${this.apiKey}`,
          'Content-Type': 'audio/wav'
        },
        body: audioBuffer
      }
    )
    
    if (!response.ok) {
      throw new Error(`Deepgram API error: ${response.status}`)
    }
    
    const data = await response.json()
    
    return {
      transcript: data.results.channels[0].alternatives[0].transcript,
      confidence: data.results.channels[0].alternatives[0].confidence,
      metadata: {
        duration: data.metadata.duration,
        language: data.metadata.detected_language || options.language
      }
    }
  }
}

interface TranscriptionResult {
  transcript: string
  confidence: number
  metadata: {
    duration: number
    language: string
  }
}
```

---

## 🖥️ Desktop Client Implementation

### **1. Tauri Projekt Setup**

```bash
# Create Tauri Project
pnpm create tauri-app

# Projekt-Name: voicetranscriber-desktop
# Template: React + TypeScript
# Package Manager: pnpm

cd voicetranscriber-desktop

# Dependencies installieren
pnpm add @tanstack/react-query zustand
pnpm add -D @types/node
```

### **2. Tauri Configuration**

```json
// src-tauri/tauri.conf.json
{
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  },
  "package": {
    "productName": "VoiceTranscriber",
    "version": "1.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "open": true
      },
      "dialog": {
        "all": true
      },
      "globalShortcut": {
        "all": true
      },
      "clipboard": {
        "all": true,
        "writeText": true,
        "readText": true
      }
    },
    "systemTray": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true,
      "menuOnLeftClick": false
    },
    "windows": [
      {
        "title": "VoiceTranscriber Settings",
        "width": 600,
        "height": 500,
        "resizable": true,
        "fullscreen": false,
        "decorations": true,
        "visible": false,
        "skipTaskbar": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; connect-src https://voice.quievreux.de"
    }
  }
}
```

### **3. Rust Backend (Tauri Commands)**

```rust
// src-tauri/src/main.rs
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{
    CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu,
    Manager, GlobalShortcutManager
};
use enigo::{Enigo, Key, KeyboardControllable};
use keyring::Entry;

mod audio;
mod api;

#[derive(Clone, serde::Serialize)]
struct RecordingState {
    is_recording: bool,
}

fn main() {
    // System Tray Menu
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let tray_menu = SystemTrayMenu::new()
        .add_item(settings)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(quit);
    
    let system_tray = SystemTray::new().with_menu(tray_menu);
    
    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "settings" => {
                        let window = app.get_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .setup(|app| {
            // Register Global Hotkey
            let mut shortcut = app.global_shortcut_manager();
            shortcut
                .register("CmdOrCtrl+Shift+V", move || {
                    // Trigger recording
                    println!("Hotkey pressed!");
                })
                .unwrap();
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            transcribe_audio,
            inject_text,
            save_credentials,
            get_credentials
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn start_recording(app_handle: tauri::AppHandle) -> Result<(), String> {
    app_handle.emit_all("recording-started", RecordingState { is_recording: true })
        .map_err(|e| e.to_string())?;
    
    audio::start_recording().await
}

#[tauri::command]
async fn stop_recording() -> Result<Vec<u8>, String> {
    audio::stop_recording().await
}

#[tauri::command]
async fn transcribe_audio(
    audio_data: Vec<u8>,
    access_token: String
) -> Result<String, String> {
    api::transcribe(audio_data, &access_token).await
}

#[tauri::command]
fn inject_text(text: String) -> Result<(), String> {
    let mut enigo = Enigo::new();
    enigo.key_sequence(&text);
    Ok(())
}

#[tauri::command]
fn save_credentials(access_token: String, refresh_token: String) -> Result<(), String> {
    let entry = Entry::new("VoiceTranscriber", "tokens")
        .map_err(|e| e.to_string())?;
    
    let credentials = format!("{}|{}", access_token, refresh_token);
    entry.set_password(&credentials).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn get_credentials() -> Result<(String, String), String> {
    let entry = Entry::new("VoiceTranscriber", "tokens")
        .map_err(|e| e.to_string())?;
    
    let credentials = entry.get_password().map_err(|e| e.to_string())?;
    let parts: Vec<&str> = credentials.split('|').collect();
    
    if parts.len() != 2 {
        return Err("Invalid credentials format".to_string());
    }
    
    Ok((parts[0].to_string(), parts[1].to_string()))
}
```

### **4. Audio Capture Modul**

```rust
// src-tauri/src/audio.rs
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

static RECORDING_BUFFER: once_cell::sync::Lazy<Arc<Mutex<Vec<f32>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

pub async fn start_recording() -> Result<(), String> {
    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or("No input device available")?;
    
    let config = device.default_input_config()
        .map_err(|e| e.to_string())?;
    
    let buffer = RECORDING_BUFFER.clone();
    buffer.lock().unwrap().clear();
    
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            buffer.lock().unwrap().extend_from_slice(data);
        },
        |err| eprintln!("Error: {}", err),
        None
    ).map_err(|e| e.to_string())?;
    
    stream.play().map_err(|e| e.to_string())?;
    
    // Store stream in static to keep it alive
    std::mem::forget(stream);
    
    Ok(())
}

pub async fn stop_recording() -> Result<Vec<u8>, String> {
    // Get recorded samples
    let samples = RECORDING_BUFFER.lock().unwrap().clone();
    
    // Convert to WAV
    let wav_bytes = samples_to_wav(&samples)?;
    
    Ok(wav_bytes)
}

fn samples_to_wav(samples: &[f32]) -> Result<Vec<u8>, String> {
    use hound::{WavSpec, WavWriter};
    
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut cursor = std::io::Cursor::new(Vec::new());
    let mut writer = WavWriter::new(&mut cursor, spec)
        .map_err(|e| e.to_string())?;
    
    for &sample in samples {
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16).map_err(|e| e.to_string())?;
    }
    
    writer.finalize().map_err(|e| e.to_string())?;
    Ok(cursor.into_inner())
}
```

### **5. API Client Modul**

```rust
// src-tauri/src/api.rs
use reqwest::multipart;

const API_BASE_URL: &str = "https://voice.quievreux.de/api";

pub async fn transcribe(
    audio_data: Vec<u8>,
    access_token: &str
) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let form = multipart::Form::new()
        .part(
            "audio",
            multipart::Part::bytes(audio_data)
                .file_name("recording.wav")
                .mime_str("audio/wav")
                .map_err(|e| e.to_string())?
        );
    
    let response = client
        .post(&format!("{}/transcribe", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        let error_body = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error: {}", error_body));
    }
    
    let result: TranscriptionResponse = response
        .json()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(result.text)
}

#[derive(serde::Deserialize)]
struct TranscriptionResponse {
    text: String,
    confidence: f32,
    duration: f32,
}
```

### **6. Frontend (React)**

```typescript
// src/App.tsx
import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

interface RecordingState {
  isRecording: boolean
}

function App() {
  const [isRecording, setIsRecording] = useState(false)
  const [isAuthenticated, setIsAuthenticated] = useState(false)
  const [status, setStatus] = useState('Ready')
  
  useEffect(() => {
    // Check for saved credentials
    checkAuth()
    
    // Listen for recording events
    const unlisten = listen<RecordingState>('recording-started', (event) => {
      setIsRecording(event.payload.isRecording)
    })
    
    return () => {
      unlisten.then(fn => fn())
    }
  }, [])
  
  async function checkAuth() {
    try {
      const [accessToken, refreshToken] = await invoke<[string, string]>(
        'get_credentials'
      )
      
      if (accessToken && refreshToken) {
        setIsAuthenticated(true)
      }
    } catch (error) {
      console.log('No saved credentials')
    }
  }
  
  async function handleRecord() {
    if (isRecording) {
      setStatus('Processing...')
      
      try {
        // Stop recording and get audio data
        const audioData = await invoke<number[]>('stop_recording')
        
        // Get access token
        const [accessToken] = await invoke<[string, string]>('get_credentials')
        
        // Transcribe
        const text = await invoke<string>('transcribe_audio', {
          audioData: Array.from(audioData),
          accessToken
        })
        
        // Inject text
        await invoke('inject_text', { text })
        
        setStatus('Done!')
        setTimeout(() => setStatus('Ready'), 2000)
        
      } catch (error) {
        setStatus(`Error: ${error}`)
      } finally {
        setIsRecording(false)
      }
      
    } else {
      setStatus('Recording...')
      await invoke('start_recording')
      setIsRecording(true)
    }
  }
  
  if (!isAuthenticated) {
    return <LoginScreen onLogin={() => setIsAuthenticated(true)} />
  }
  
  return (
    <div className="container">
      <h1>VoiceTranscriber</h1>
      <div className="status">{status}</div>
      
      <button 
        onClick={handleRecord}
        className={isRecording ? 'recording' : ''}
      >
        {isRecording ? '⏹ Stop' : '🎤 Record'}
      </button>
      
      <p className="hint">
        Or press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>V</kbd>
      </p>
    </div>
  )
}
```

---

## 🔒 Sicherheitskonzept

### **1. Client-Side Security**

```rust
// Credentials im System Keychain speichern (nie im Klartext!)
use keyring::Entry;

fn save_token_secure(token: &str) -> Result<(), String> {
    let entry = Entry::new("VoiceTranscriber", "access_token")?;
    entry.set_password(token)?;
    Ok(())
}

// HTTPS Pinning (Optional, aber empfohlen)
use reqwest::Certificate;

fn create_pinned_client() -> reqwest::Client {
    let cert = include_bytes!("../certs/voice.quievreux.de.pem");
    let cert = Certificate::from_pem(cert).unwrap();
    
    reqwest::Client::builder()
        .add_root_certificate(cert)
        .build()
        .unwrap()
}
```

### **2. Server-Side Security Checklist**

```typescript
// app/middleware.ts
import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'

export function middleware(request: NextRequest) {
  // Security Headers
  const headers = new Headers(request.headers)
  headers.set('X-Content-Type-Options', 'nosniff')
  headers.set('X-Frame-Options', 'DENY')
  headers.set('X-XSS-Protection', '1; mode=block')
  headers.set('Referrer-Policy', 'strict-origin-when-cross-origin')
  
  // CORS (nur für Desktop-Client)
  if (request.headers.get('origin') === 'tauri://localhost') {
    headers.set('Access-Control-Allow-Origin', 'tauri://localhost')
    headers.set('Access-Control-Allow-Methods', 'POST, GET, OPTIONS')
    headers.set('Access-Control-Allow-Headers', 'Authorization, Content-Type')
  }
  
  // Rate Limiting Headers
  const response = NextResponse.next({
    request: {
      headers,
    },
  })
  
  return response
}

export const config = {
  matcher: '/api/:path*',
}
```

### **3. Input Validation**

```typescript
// Immer mit Zod validieren!
import { z } from 'zod'

const transcribeSchema = z.object({
  audio: z.instanceof(File)
    .refine(file => file.size <= 10 * 1024 * 1024, 'Max 10MB')
    .refine(
      file => ['audio/wav', 'audio/webm', 'audio/ogg'].includes(file.type),
      'Invalid audio format'
    )
})

// Usage
try {
  const { audio } = transcribeSchema.parse({ audio: formData.get('audio') })
} catch (error) {
  return Response.json({ error: 'Invalid input' }, { status: 400 })
}
```

---

## 📊 Monitoring & Analytics

### **1. Vercel Analytics Setup**

```typescript
// app/layout.tsx
import { Analytics } from '@vercel/analytics/react'

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        {children}
        <Analytics />
      </body>
    </html>
  )
}
```

### **2. Custom Metrics Tracking**

```typescript
// lib/metrics.ts
import { kv } from '@vercel/kv'

export async function trackMetric(
  metric: string,
  value: number,
  tags?: Record<string, string>
) {
  const timestamp = Date.now()
  const key = `metrics:${metric}:${getDay()}`
  
  await kv.lpush(key, {
    value,
    timestamp,
    tags
  })
  
  // Auto-expire after 30 days
  await kv.expire(key, 30 * 24 * 60 * 60)
}

// Usage
await trackMetric('transcription.duration', result.duration, {
  tier: user.tier,
  language: 'de'
})
```

### **3. Error Tracking**

```bash
# Sentry Integration
pnpm add @sentry/nextjs

# Initialisierung
```

```typescript
// sentry.client.config.ts
import * as Sentry from '@sentry/nextjs'

Sentry.init({
  dsn: process.env.NEXT_PUBLIC_SENTRY_DSN,
  tracesSampleRate: 1.0,
  environment: process.env.VERCEL_ENV || 'development'
})
```

---

## 🚀 Deployment Workflow

### **1. Backend Deployment**

```bash
# Production Deployment
git push origin main
# → Auto-Deploy via Vercel GitHub Integration

# Preview Deployments (für jeden Branch)
git push origin feature/new-feature
# → https://voicetranscriber-git-feature-new-feature.vercel.app

# Manual Deploy
vercel --prod
```

### **2. Desktop App Build**

```bash
# Windows Installer
cd voicetranscriber-desktop
pnpm tauri build --target x86_64-pc-windows-msvc

# Output:
# src-tauri/target/release/bundle/msi/VoiceTranscriber_1.0.0_x64_en-US.msi
# src-tauri/target/release/bundle/nsis/VoiceTranscriber_1.0.0_x64-setup.exe

# macOS
pnpm tauri build --target aarch64-apple-darwin  # M1/M2
pnpm tauri build --target x86_64-apple-darwin   # Intel

# Linux
pnpm tauri build --target x86_64-unknown-linux-gnu
```

### **3. GitHub Actions CI/CD**

```yaml
# .github/workflows/desktop-release.yml
name: Release Desktop App

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    strategy:
      matrix:
        platform: [macos-latest, ubuntu-latest, windows-latest]
    
    runs-on: ${{ matrix.platform }}
    
    steps:
      - uses: actions/checkout@v4
      
      - uses: pnpm/action-setup@v2
        with:
          version: 8
      
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'pnpm'
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install dependencies
        run: pnpm install
      
      - name: Build Tauri App
        run: pnpm tauri build
      
      - name: Upload Release
        uses: softprops/action-gh-release@v1
        with:
          files: src-tauri/target/release/bundle/**/*
```

---

## 💰 Kosten-Rechnung (Real Numbers)

### **Monat 1-3: MVP Phase (0-500 Users)**

```
KOSTEN:
├─ Vercel Hobby: €0
├─ Deepgram API: 500 Users × 660 Min × $0.0043 = $1,419
└─ TOTAL: $1,419/Monat

REVENUE (50% paying):
├─ 250 × €9 Basic = €2,250
└─ Profit: €2,250 - $1,419 = ~€800/Monat
```

### **Monat 4-12: Growth (500-5,000 Users)**

```
KOSTEN:
├─ Vercel Pro: €20/Monat
├─ Deepgram API: 5,000 × 660 × $0.0043 = $14,190
└─ TOTAL: $14,210/Monat

REVENUE (50% paying):
├─ 2,000 Basic × €9 = €18,000
├─ 500 Pro × €29 = €14,500
└─ TOTAL: €32,500

Profit: €32,500 - $14,210 = ~€18,000/Monat
```

### **Jahr 2: Scale (5,000-20,000 Users)**

```
KOSTEN:
├─ Vercel Pro: €60/Monat (3× Teams)
├─ Deepgram Enterprise Deal: $0.0036/Min
├─ 20,000 × 660 × $0.0036 = $47,520
└─ TOTAL: $47,580/Monat

REVENUE (60% paying):
├─ 8,000 Basic × €9 = €72,000
├─ 4,000 Pro × €29 = €116,000
└─ TOTAL: €188,000/Monat

Profit: €188,000 - $47,580 = ~€140,000/Monat
```

---

## 📋 Launch Checklist

### **Pre-Launch**

- [ ] Vercel Projekt erstellt
- [ ] KV Database konfiguriert
- [ ] Environment Variables gesetzt
- [ ] Deepgram Account + API Key
- [ ] Domain konfiguriert (voice.quievreux.de)
- [ ] SSL-Zertifikat verifiziert
- [ ] Tauri Desktop App kompiliert
- [ ] Windows Installer getestet
- [ ] macOS App signiert (Apple Developer Account)

### **Launch Day**

- [ ] Marketing Landing Page live
- [ ] Desktop App Download verfügbar
- [ ] Stripe Payment Integration aktiv
- [ ] Support Email eingerichtet
- [ ] Monitoring Dashboard läuft
- [ ] Error Tracking (Sentry) aktiv

### **Post-Launch (Woche 1)**

- [ ] Erste 10 User onboarded
- [ ] Bug Reports gesammelt
- [ ] Performance Metrics analysiert
- [ ] Kostenüberwachung eingerichtet
- [ ] Feedback-Loop etabliert

---

## 🎯 Success Metrics

### **Technische KPIs**

```
API Performance:
├─ P50 Latenz: <500ms ✅
├─ P95 Latenz: <2s ✅
├─ Error Rate: <0.5% ✅
└─ Uptime: >99.9% ✅

Transkriptions-Qualität:
├─ WER (Word Error Rate): <10% ✅
├─ User Satisfaction: >4.5/5 ✅
└─ Retry Rate: <5% ✅
```

### **Business KPIs**

```
Conversion Funnel:
├─ Download → Registration: >60%
├─ Registration → First Use: >80%
├─ Free → Paid: >30% (nach 30 Tagen)
└─ Churn Rate: <5%/Monat

Revenue:
├─ MRR Growth: +20%/Monat
├─ ARPU: €15/Monat
└─ LTV/CAC Ratio: >3:1
```

---

## 📚 Nächste Schritte

### **Week 1-2: Backend Setup**
1. Vercel Projekt erstellen
2. Authentication API implementieren
3. Transcription API implementieren
4. Rate Limiting testen

### **Week 3-4: Desktop Client**
1. Tauri Projekt setup
2. Audio Capture implementieren
3. API Integration
4. System Tray + Hotkeys

### **Week 5: Testing & Polish**
1. End-to-End Tests
2. Performance Optimierung
3. Error Handling
4. User Onboarding

### **Week 6: Launch**
1. Beta Tester (20 User)
2. Feedback sammeln
3. Bugfixes
4. Public Launch

---

## 🔗 Ressourcen

- **Vercel Docs:** https://vercel.com/docs
- **Tauri Docs:** https://tauri.app/v1/guides/
- **Deepgram API:** https://developers.deepgram.com
- **Upstash Rate Limiting:** https://upstash.com/docs/redis/sdks/ratelimit-ts/overview

---

**Ready to build? Let's ship this! 🚀**
