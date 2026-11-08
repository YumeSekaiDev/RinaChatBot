use crate::state::{AppState, VoiceDto};
use std::io::Cursor;
use std::process::{Child, Command};
use std::sync::atomic::Ordering;

// ── GPT-SoVITS Rina voice clone settings ─────────────────────────────────────
/// Base URL for the GPT-SoVITS api_v2.py server (started separately via start-rina-tts.sh)
const GPTSOVITS_BASE: &str = "http://127.0.0.1:9880";

/// Reference audio clip for Rina's voice — a clean 4-second Japanese line
const RINA_REF_AUDIO: &str = "/mnt/UNTV News/Game Dev/VoiceChatRina/JapaneseVoice/sliced/Rina Voice.wav_0000532480_0000678720.wav";

/// What Rina says in that reference clip
const RINA_REF_TEXT: &str = "りなちゃんボード";

/// Language code used for the reference clip prompt
const RINA_REF_LANG: &str = "ja";
// ─────────────────────────────────────────────────────────────────────────────

/// Clean text before passing to TTS so markdown syntax and code blocks aren't read aloud.
pub fn clean_text_for_speech(raw: &str) -> String {
    let mut text = raw.to_string();

    // Remove code blocks ```...```
    while let Some(start) = text.find("```") {
        if let Some(end) = text[start + 3..].find("```") {
            text.replace_range(start..start + 3 + end + 3, " [code snippet omitted] ");
        } else {
            text.replace_range(start.., "");
            break;
        }
    }

    // Remove inline backticks `code`
    text = text.replace('`', "");

    // Remove bold/italics asterisks and underscores
    text = text.replace("**", "");
    text = text.replace("__", "");
    text = text.replace('*', "");

    // Remove markdown headers
    let lines: Vec<&str> = text
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                trimmed.trim_start_matches('#').trim_start()
            } else {
                line
            }
        })
        .collect();
    text = lines.join(" ");

    // Remove markdown links [title](url) -> title
    while let Some(start_bracket) = text.find('[') {
        if let Some(end_bracket) = text[start_bracket..].find(']') {
            let link_title = &text[start_bracket + 1..start_bracket + end_bracket];
            let rest = &text[start_bracket + end_bracket + 1..];
            if rest.starts_with('(') {
                if let Some(end_paren) = rest.find(')') {
                    let total_len = end_bracket + 1 + end_paren + 1;
                    let title_owned = link_title.to_string();
                    text.replace_range(start_bracket..start_bracket + total_len, &title_owned);
                    continue;
                }
            }
        }
        break;
    }

    // Collapse multiple spaces
    let words: Vec<&str> = text.split_whitespace().collect();
    words.join(" ")
}

/// Check if the GPT-SoVITS server is reachable at GPTSOVITS_BASE.
pub fn is_gptsovits_running() -> bool {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build();
    match client {
        Ok(c) => c.get(&format!("{}/", GPTSOVITS_BASE)).send().is_ok(),
        Err(_) => false,
    }
}

pub fn list_voices(state: &AppState) -> Result<Vec<VoiceDto>, String> {
    let mut voice_dtos = Vec::new();

    // Rina-chan cloned voice — shown first when GPT-SoVITS server is running
    if is_gptsovits_running() {
        voice_dtos.push(VoiceDto {
            id: "rina_gptsovits".to_string(),
            name: "🌸 Rina Tennoji (Voice Clone)".to_string(),
            language: "ja".to_string(),
        });
    }

    // Default entry — young female voice (18-year-old girl, airy & bright)
    voice_dtos.push(VoiceDto {
        id: "default".to_string(),
        name: "Young Female Voice (en-us+f5)".to_string(),
        language: "en-us".to_string(),
    });

    if let Ok(lock) = state.tts_instance.lock() {
        if let Some(ref tts) = *lock {
            if let Ok(voices) = tts.voices() {
                for v in voices {
                    voice_dtos.push(VoiceDto {
                        id: v.id().clone(),
                        name: v.name().clone(),
                        language: "system".to_string(),
                    });
                }
            }
        }
    }

    // If voices list is still just default on Linux, try checking espeak-ng voices
    if cfg!(target_os = "linux") && voice_dtos.len() <= 1 {
        if let Ok(output) = Command::new("espeak-ng").arg("--voices").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let lang = parts[1];
                    let name = parts[3];
                    voice_dtos.push(VoiceDto {
                        id: name.to_string(),
                        name: format!("{} ({})", name, lang),
                        language: lang.to_string(),
                    });
                }
            }
        }
    }

    Ok(voice_dtos)
}

pub fn speak(
    state: &AppState,
    raw_text: &str,
    voice_id: Option<String>,
    speed: f32,
    volume: f32,
    use_edge_fallback: bool,
) -> Result<(), String> {
    stop_all_speech(state)?;

    let clean = clean_text_for_speech(raw_text);
    if clean.is_empty() {
        return Ok(());
    }

    state.set_speaking(true);

    // Option 1: GPT-SoVITS Rina voice clone — primary path when server is running
    let use_rina_voice = matches!(voice_id.as_deref(), Some("rina_gptsovits") | None);
    if use_rina_voice && is_gptsovits_running() {
        match speak_with_gptsovits(state, &clean, speed, volume) {
            Ok(_) => {
                state.set_speaking(false);
                return Ok(());
            }
            Err(e) => {
                eprintln!("[GPT-SoVITS] Error: {}. Falling back to system TTS.", e);
            }
        }
    }

    // Option 2: Edge-TTS Fallback if explicitly requested or local TTS not available
    if use_edge_fallback {
        match speak_with_edge_tts(state, &clean, volume) {
            Ok(_) => {
                state.set_speaking(false);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Edge-TTS fallback error: {}, reverting to platform TTS", e);
            }
        }
    }

    // Option 2: Platform Native TTS via tts crate (Windows SAPI only)
    // NOTE: Skipped on Linux — speech-dispatcher defaults to a male voice and ignores our
    //       voice selection. We use espeak-ng directly (Option 3) for full voice control.
    #[cfg(not(target_os = "linux"))]
    let mut tts_succeeded = false;
    #[cfg(not(target_os = "linux"))]
    {
        if let Ok(mut lock) = state.tts_instance.lock() {
            if let Some(ref mut tts) = *lock {
                if let Some(ref vid) = voice_id {
                    if vid != "default" {
                        if let Ok(voices) = tts.voices() {
                            if let Some(target) = voices.iter().find(|v| v.id() == *vid) {
                                let _ = tts.set_voice(target);
                            }
                        }
                    }
                }

                // Set speed rate (0.5 to 2.0)
                let _ = tts.set_rate(speed);
                let _ = tts.set_volume(volume);

                if let Ok(_) = tts.speak(&clean, false) {
                    tts_succeeded = true;
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    if tts_succeeded {
        // Wait briefly for speech to proceed, or let asynchronous system handle it
        return Ok(());
    }

    // Option 3: On Linux -> espeak-ng / espeak CLI (primary path, full voice control)
    if cfg!(target_os = "linux") {
        let wpm = (175.0 * speed).clamp(80.0, 450.0) as i32;
        let vol_int = (volume * 100.0).clamp(0.0, 100.0) as i32;

        let espeak_bin = if Command::new("espeak-ng").arg("--version").output().is_ok() {
            "espeak-ng"
        } else {
            "espeak"
        };

        let mut cmd = Command::new(espeak_bin);
        cmd.arg("-s").arg(wpm.to_string());
        cmd.arg("-a").arg(vol_int.to_string());

        // Use provided voice or fall back to a young female English voice by default
        let voice_arg = match voice_id.as_deref() {
            Some(vid) if vid != "default" => vid.to_string(),
            _ => "en-us+f5".to_string(), // Young female voice default (18-year-old girl, airy & bright)
        };
        cmd.arg("-v").arg(&voice_arg);

        cmd.arg(&clean);

        match cmd.spawn() {
            Ok(child) => {
                *state.current_process.lock().unwrap() = Some(child);
                return Ok(());
            }
            Err(e) => {
                eprintln!("espeak execution failed: {}", e);
            }
        }
    }

    // Option 4: Fallback on Windows -> PowerShell SAPI
    if cfg!(target_os = "windows") {
        let rate_int = ((speed - 1.0) * 5.0).clamp(-10.0, 10.0) as i32;
        let vol_int = (volume * 100.0).clamp(0.0, 100.0) as i32;

        let escaped = clean.replace('"', "\"\"").replace('`', "``").replace('$', "`$");
        let ps_script = format!(
            "$s = New-Object -ComObject SAPI.SpVoice; $s.Rate = {}; $s.Volume = {}; $s.Speak(\"{}\");",
            rate_int, vol_int, escaped
        );

        match Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .spawn()
        {
            Ok(child) => {
                *state.current_process.lock().unwrap() = Some(child);
                return Ok(());
            }
            Err(e) => {
                eprintln!("PowerShell SAPI execution failed: {}", e);
            }
        }
    }

    state.set_speaking(false);
    Err("Text-to-Speech could not produce audio on this system. Please check system speech settings.".to_string())
}

pub fn stop_all_speech(state: &AppState) -> Result<(), String> {
    state.set_speaking(false);

    // 1. Stop tts crate speech
    if let Ok(mut lock) = state.tts_instance.lock() {
        if let Some(ref mut tts) = *lock {
            let _ = tts.stop();
        }
    }

    // 2. Kill spawned subprocess (espeak-ng or powershell)
    if let Ok(mut proc_lock) = state.current_process.lock() {
        if let Some(mut child) = proc_lock.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    // 3. Stop Rodio sink
    if let Ok(sink_lock) = state.rodio_sink.lock() {
        if let Some(ref sink) = *sink_lock {
            sink.stop();
        }
    }

    Ok(())
}

/// Fallback online Edge-TTS via HTTP synthesizer played through rodio
fn speak_with_edge_tts(state: &AppState, text: &str, volume: f32) -> Result<(), String> {
    // URL-encode text for speech query
    let encoded: String = text
        .chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect();

    // Standard public Google Translate / Edge TTS fallback endpoint for audio
    let url = format!(
        "https://translate.google.com/translate_tts?ie=UTF-8&tl=en&client=tw-ob&q={}",
        encoded
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .map_err(|e| format!("Edge-TTS request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Edge-TTS returned status {}", response.status()));
    }

    let bytes = response.bytes().map_err(|e| e.to_string())?;

    // Play through rodio sink
    let sink_lock = state.rodio_sink.lock().map_err(|e| e.to_string())?;
    if let Some(ref sink) = *sink_lock {
        sink.set_volume(volume);
        let cursor = Cursor::new(bytes);
        if let Ok(source) = rodio::Decoder::new(cursor) {
            sink.append(source);
            sink.sleep_until_end();
            return Ok(());
        }
    }

    Err("Rodio audio playback device unavailable".to_string())
}

/// Call the GPT-SoVITS api_v2.py /tts endpoint to synthesize text with Rina's
/// cloned voice, then play the returned WAV through the existing Rodio sink.
///
/// The server must be running (start-rina-tts.sh). Falls back gracefully on error.
fn speak_with_gptsovits(state: &AppState, text: &str, speed: f32, volume: f32) -> Result<(), String> {
    // Build the JSON POST body
    let body = serde_json::json!({
        "text": text,
        "text_lang": "ja",
        "ref_audio_path": RINA_REF_AUDIO,
        "prompt_text": RINA_REF_TEXT,
        "prompt_lang": RINA_REF_LANG,
        "top_k": 15,
        "top_p": 1.0,
        "temperature": 1.0,
        "speed_factor": (speed as f64).clamp(0.5, 2.0),
        "text_split_method": "cut5",
        "batch_size": 1,
        "media_type": "wav",
        "streaming_mode": false
    });

    let client = reqwest::blocking::Client::builder()
        // CPU inference can be slow; allow up to 120 seconds
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("HTTP client build error: {}", e))?;

    let response = client
        .post(&format!("{}/tts", GPTSOVITS_BASE))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .map_err(|e| format!("GPT-SoVITS request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let err_body = response.text().unwrap_or_default();
        return Err(format!("GPT-SoVITS returned HTTP {}: {}", status, err_body));
    }

    let bytes = response.bytes().map_err(|e| e.to_string())?;
    if bytes.is_empty() {
        return Err("GPT-SoVITS returned empty audio".to_string());
    }

    // Play through Rodio sink (same path used by Edge-TTS)
    let sink_lock = state.rodio_sink.lock().map_err(|e| e.to_string())?;
    if let Some(ref sink) = *sink_lock {
        sink.set_volume(volume);
        let cursor = Cursor::new(bytes);
        match rodio::Decoder::new(cursor) {
            Ok(source) => {
                sink.append(source);
                sink.sleep_until_end();
                return Ok(());
            }
            Err(e) => {
                return Err(format!("Rodio decode error for GPT-SoVITS audio: {}", e));
            }
        }
    }

    Err("Rodio audio playback device unavailable for GPT-SoVITS output".to_string())
}
