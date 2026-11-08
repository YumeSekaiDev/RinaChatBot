# 🎙️ VoiceChatRina (100% Local Desktop Assistant)

A complete, private local voice chatbot application built in **Rust** and **Tauri 2.0** that you can speak to out loud, powered by **Ollama** and **Whisper STT**.

Works seamlessly on both **Windows 11** and **Fedora Linux**.

---

## ✨ Features

- **100% Local & Private**: No cloud APIs, no telemetry, no API keys required. Works completely offline.
- **Natural Voice Interaction**:
  - **Push-to-Talk with Spacebar**: Hold `[SPACE]` anywhere in the app to speak, release to send immediately.
  - **Big Talk Button**: Click to start recording, click again to stop and process.
  - **Speech-to-Text**: High-accuracy local transcription using `whisper-rs` (Whisper.cpp) with `ggml-tiny.en.bin` (75MB ultra-fast lightweight model for laptops, auto-downloaded on first run; also supports `ggml-base.en.bin`).
  - **Text-to-Speech**:
    - **Windows**: Windows SAPI via `tts` crate and native COM voice synthesis.
    - **Linux**: Speech-Dispatcher and `espeak-ng` command line.
    - **Fallback**: Neural Edge-TTS via HTTP if online.
  - **Animated Status Indicator**:
    - **Listening**: Pulsing red halo with waveform visualizer.
    - **Thinking**: Pulsing blue radar with LLM processing state.
    - **Speaking**: Pulsing green audio spectrum equalizer.
- **Local LLM Intelligence (Ollama: 0.3B to 7B Models)**:
  - Supports a wide spectrum of models from **0.3B to 7B**:
    - **Ultra-Light (0.3B ~ 0.5B)**: `smollm2:360m` (~230MB), `qwen2.5:0.5b` (~398MB, Default Fast), `qwen2:0.5b` (~350MB).
    - **Compact (1B ~ 2B)**: `llama3.2:1b` (~1.3GB), `qwen2.5:1.5b` (~986MB), `deepseek-r1:1.5b` (~1.1GB), `gemma2:2b` (~1.6GB).
    - **Balanced (3B ~ 4B)**: `llama3.2:3b` (~2.0GB), `qwen2.5:3b` (~1.9GB), `phi3.5:3.8b` (~2.2GB).
    - **High-IQ (7B ~ 8B)**: `qwen2.5:7b` (~4.7GB), `mistral:7b` (~4.1GB), `deepseek-r1:7b` (~4.7GB), `llama3.1:8b` (~4.7GB).
  - Auto-lists models from `GET /api/tags`.
  - In-app 1-click model puller with preset chips to download models without typing.
  - Configurable system prompt and temperature slider (0.0 to 1.0).
- **Audio & Voice Settings**:
  - Microphone selector dropdown (enumerated via `cpal`).
  - Voice selector dropdown (system voices).
  - Speech speed slider (0.5x to 2.0x).
  - Volume slider (0% to 100%).
  - Auto-speak replies toggle (ON / OFF).
  - Stop Speaking button (`[⏹ Stop Speaking]`) to instantly interrupt speech.
- **Data & History**:
  - Auto-saves conversations to `chat_history.json` in the application data folder.
  - Clear chat button.
  - Export conversation to a formatted `.txt` transcript file.
- **Robust Error Handling**:
  - Friendly alerts when Ollama is offline: *"Ollama is not running. Run 'ollama serve'"*.
  - Prompt to pull `qwen2.5:0.5b` if missing.
  - Friendly warnings when no microphone is detected.

---

## 📁 Project Structure

```
C:\Project\
├── src\                       # Frontend (Pure modern HTML/CSS/JS, zero Node runtime needed)
│   ├── index.html             # ChatGPT-style UI with status pill & voice dock
│   ├── styles.css             # Futuristic dark HUD theme with pulsing state animations
│   └── app.js                 # Push-to-talk, Ollama streaming, and Tauri IPC handling
├── src-tauri\                 # Rust Tauri 2.0 Backend
│   ├── src\
│   │   ├── main.rs            # Application entry point & command registration
│   │   ├── state.rs           # Thread-safe application state & DTOs
│   │   ├── audio.rs           # CPAL microphone capture, channel downmix & resampling
│   │   ├── stt.rs             # Whisper-rs STT & auto-download for ggml-base.en.bin
│   │   ├── tts.rs             # Cross-platform TTS (SAPI, Speech-Dispatcher, espeak-ng, Edge-TTS)
│   │   ├── ollama.rs          # Ollama REST API client (tags, chat, pull)
│   │   └── history.rs         # Chat history persistence (chat_history.json) and export
│   ├── icons\                 # App icons (32x32, 128x128, 256x256, 512x512, .ico, .icns)
│   ├── capabilities\
│   │   └── default.json       # Tauri 2.0 permission capabilities
│   ├── Cargo.toml             # Rust dependencies (tauri, cpal, rodio, whisper-rs, tts, reqwest)
│   ├── build.rs               # Tauri build script
│   └── tauri.conf.json        # Tauri v2 configuration (identifier: com.yume.jarvis)
├── tauri.conf.json            # Root configuration link
├── build-windows.bat          # Automated Windows build script (.msi installer)
├── build-rpm.sh               # Automated Fedora Linux build script (.rpm package)
├── Instructions.txt           # Specification guidelines
└── README.md                  # Documentation
```

---

## 🚀 Quick Start Guide

### 1. Prerequisites (Ollama)

Ensure Ollama is installed and running on your system:
```bash
# Start the Ollama server
ollama serve

# Pull the fast 0.5B model (only ~398 MB, runs blazingly fast on any laptop)
ollama pull qwen2.5:0.5b
```

---

### 2. Building on Windows 11

To build the application and generate the Windows installer (`.msi` and `.exe`):

1. Double-click or run:
   ```cmd
   build-windows.bat
   ```
2. The script will:
   - Check and install Rust / Cargo if missing.
   - Install `tauri-cli` (`cargo-tauri`).
   - Compile the release binary and bundle.
   - Produce the installer in:
     `src-tauri\target\release\bundle\msi\` and `src-tauri\target\release\bundle\nsis\`.

Alternatively, with Cargo installed:
```cmd
cargo tauri build
```

---

### 3. Building on Fedora Linux

To build the application and generate the RPM package (`.rpm`):

1. Make the script executable and run:
   ```bash
   chmod +x build-rpm.sh
   ./build-rpm.sh
   ```
2. The script will:
   - Install all required `dnf` build dependencies (`webkit2gtk4.1-devel`, `gtk3-devel`, `alsa-lib-devel`, `speech-dispatcher-devel`, `espeak-ng`, `rpm-build`, etc.).
   - Verify Rust and Tauri CLI.
   - Compile and package the RPM in:
     `src-tauri/target/release/bundle/rpm/*.rpm`.
3. To install the generated RPM:
   ```bash
   sudo dnf install ./src-tauri/target/release/bundle/rpm/*.rpm
   ```

---

## ⌨️ Shortcuts & Controls

| Action | Control |
|---|---|
| **Hold to Talk** | Press & hold `[SPACEBAR]` anywhere (release to send) |
| **Click to Talk** | Click `[🎤]` button to record; click again to stop & send |
| **Stop Speech / Mute** | Click `[⏹ Stop Speaking]` button |
| **Text Chat** | Type in the bottom input bar and press `[Enter]` |
| **Settings** | Click `[⚙ Settings]` in the top header |
| **Export Chat** | Click `[Export]` in the top header |
| **Clear Chat** | Click `[Clear]` in the top header |
# RinaChatBot
