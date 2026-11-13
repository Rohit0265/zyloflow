d# ZyloFlow — AI-Powered Voice-to-Text Desktop Application

ZyloFlow is a lightweight, premium-looking, AI-powered voice-to-text desktop application inspired by Wispr Flow, specifically engineered for the Linux desktop (Ubuntu X11 & Wayland).

It allows users to dictate text into any focused input across their entire system using a configurable global keyboard shortcut. Recorded audio is converted into text via ultra-fast Whisper speech-to-text engines, polished and cleaned using state-of-the-art LLMs, and injected directly into the active application.

---

## Key Features

- **Global Desktop Dictation**: Press `Ctrl + Alt + Space` (configurable) in any app — VS Code, terminal, browser, Slack, LibreOffice — to dictate.
- **Background Mode & System Tray**: Runs silently in the background with a system tray menu without opening the dashboard window.
- **Floating Overlay Popup**: A sleek, unobtrusive pill floating above other windows displaying real-time states: *Listening*, *Transcribing*, *Refining*, *Inserting*, and *Completed*.
- **Modular AI Architecture**:
  - **Speech-to-Text**: Groq Whisper (`whisper-large-v3-turbo`, `whisper-large-v3`) & OpenAI Whisper (`whisper-1`).
  - **Text Refinement**: Google Gemini (`gemini-2.5-flash`), Groq LLM, xAI Grok (`grok-4.1-fast`), OpenAI (`gpt-4.1-mini`).
  - Configurable refinement modes: *Minimal*, *Standard*, and *Professional*.
- **OS-Native Text Injection**: Automatically pastes into external applications while safely preserving and restoring previous clipboard contents.
- **Secure Key Storage**: API keys are securely stored using the OS-native Secret Service / Keyring integration with an encrypted vault fallback.
- **First-Time Onboarding**: Walkthrough for microphone verification, API key setup, and explicit autostart consent.
- **Transcription History**: Local history records with character and word counters, timestamps, raw vs. refined diffs, and one-click copy.

---

## Technology Stack

- **Frontend**: React 18, TypeScript, Vite, Tailwind CSS, Lucide React, Zustand.
- **Desktop Backend**: Tauri v2, Rust.
- **Plugins**: `@tauri-apps/plugin-global-shortcut`, `@tauri-apps/plugin-autostart`, `@tauri-apps/plugin-clipboard-manager`.
- **Audio**: Low-latency ALSA/PipeWire/PulseAudio capture via CPAL & Hound (16kHz 16-bit linear PCM WAV).
- **Display Compatibility**: Tested for both Wayland and X11 display servers.

---

## Project Structure

```
zyloflow/
├── src/                         # React Frontend
│   ├── app/                     # App bootstrapping
│   ├── components/
│   │   ├── Header.tsx           # Dashboard title, status & window controls
│   │   ├── FloatingPopup.tsx    # Sleek floating overlay for active dictation
│   │   └── OnboardingModal.tsx  # First-run setup modal
│   ├── pages/
│   │   ├── Dashboard.tsx        # Main dashboard with dictation controls & history
│   │   └── Settings.tsx         # AI providers, hotkeys, and general preferences
│   ├── services/
│   │   └── api.ts               # IPC interface to Tauri backend
│   ├── stores/
│   │   └── useAppStore.ts       # Global state management via Zustand
│   ├── types/
│   │   └── index.ts             # TypeScript interfaces
│   ├── App.tsx                  # Root component (routes between main & popup)
│   ├── main.tsx                 # Entrypoint
│   └── index.css                # Tailwind base and theme variables
│
├── src-tauri/                   # Rust Backend
│   ├── src/
│   │   ├── audio.rs             # Background audio recording (CPAL + Hound WAV)
│   │   ├── error.rs             # Unified error handling
│   │   ├── hotkeys.rs           # Native global shortcut registration
│   │   ├── lifecycle.rs         # Dictation pipeline orchestration
│   │   ├── popup.rs             # Floating window positioning & state events
│   │   ├── refinement.rs        # LLM text refinement (Gemini, Groq, xAI, OpenAI)
│   │   ├── secure_storage.rs    # Secret Service keyring & encrypted vault fallback
│   │   ├── settings.rs          # Settings & history file persistence
│   │   ├── text_injection.rs    # X11 & Wayland input injection & clipboard restore
│   │   ├── tray.rs              # System tray menu and lifecycle management
│   │   ├── lib.rs               # Tauri IPC command definitions & plugin setup
│   │   └── main.rs              # Application entrypoint
│   ├── capabilities/            # Tauri v2 security capabilities
│   ├── icons/                   # High-resolution application icons
│   ├── tauri.conf.json          # Tauri v2 configuration
│   └── Cargo.toml               # Rust dependencies
│
├── task.md                      # Specification document
└── package.json                 # Frontend dependencies & scripts
```

---

## Installation & Build Instructions

### Prerequisites (Ubuntu Linux)

```bash
# Node.js 20+ and npm
sudo apt update
sudo apt install -y curl build-essential pkg-config libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev libxdo-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### 1. Install Node Dependencies

```bash
cd zyloflow
npm install
```

### 2. Run in Development Mode

```bash
npm run dev
# Or to run the complete desktop app:
npm run tauri dev
```

### 3. Build Production Binary and Debian Package (.deb)

```bash
npm run tauri build
```

The generated debian installer will be located at:
`src-tauri/target/release/bundle/deb/zyloflow_0.1.0_amd64.deb`

The standalone binary will be located at:
`src-tauri/target/release/zyloflow`

### 4. Install the .deb Package

```bash
sudo dpkg -i src-tauri/target/release/bundle/deb/zyloflow_0.1.0_amd64.deb
```

---

## Global Shortcuts

| Action | Default Shortcut | Description |
|---|---|---|
| **Toggle Dictation** | `Ctrl + Alt + Space` | Press once to start recording; press again to stop, refine, and insert text. |
| **Start / Resume** | `Ctrl + Alt + S` | Resumes the application if paused without opening the main window. |
| **Stop / Pause** | `Ctrl + Alt + X` | Cancels any ongoing recording and pauses background activity. |

*Shortcuts can be customized in the Hotkeys tab inside Settings.*

---

## Linux Integration (Wayland & X11)

- **Display Server Detection**: ZyloFlow inspects `$WAYLAND_DISPLAY`, `$XDG_SESSION_TYPE`, and `$DISPLAY` to adapt its behavior.
- **Input Injection**:
  - **X11**: Uses clipboard paste simulation via `xdotool` or direct key injection.
  - **Wayland**: Injects paste events using `wtype` or `ydotool` while maintaining clipboard safety.
  - **Safe Clipboard Restoration**: Original clipboard contents before dictation are preserved and restored after paste.

---

## Troubleshooting Guide

1. **Microphone Access Denied**:
   - Check that your user is part of the `audio` group (`sudo usermod -aG audio $USER`).
   - Check that PipeWire or PulseAudio is running (`systemctl --user status pipewire`).
2. **API Key Errors (401 / 429)**:
   - Verify that your API key is correctly entered in Settings -> AI Providers. Use the **Test** button to test connectivity.
   - Groq keys can be obtained from [Groq Console](https://console.groq.com).
   - Gemini keys can be obtained from [Google AI Studio](https://aistudio.google.com).
3. **Wayland Shortcut Restrictions**:
   - Some Wayland compositors (like GNOME Wayland) restrict global shortcut interception from sandboxed apps. ZyloFlow handles this natively; if your compositor blocks arbitrary global shortcuts, you can bind a system shortcut in GNOME Settings -> Keyboard to execute `zyloflow --dictate`.

