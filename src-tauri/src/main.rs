// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod history;
mod ollama;
mod state;
mod stt;
mod tts;

use state::{AppState, ChatMessage, StatusDto, VoiceDto};
use tauri::State;

#[tauri::command]
async fn get_status(state: State<'_, AppState>) -> Result<StatusDto, String> {
    let ollama_running = ollama::check_ollama_status().await;
    let whisper_model_present = stt::is_whisper_model_downloaded();
    let is_recording = state.is_recording();
    let is_speaking = state.is_speaking();

    Ok(StatusDto {
        ollama_running,
        whisper_model_present,
        is_recording,
        is_speaking,
    })
}

#[tauri::command]
fn get_microphones() -> Result<Vec<String>, String> {
    audio::list_input_devices()
}

#[tauri::command]
fn start_recording(state: State<'_, AppState>, device_name: Option<String>) -> Result<(), String> {
    audio::start_mic_recording(&state, device_name)
}

#[tauri::command]
async fn stop_recording_and_transcribe(state: State<'_, AppState>) -> Result<String, String> {
    let audio_samples = audio::stop_mic_recording(&state)?;

    // Run Whisper STT in a blocking thread to avoid blocking the async runtime
    tokio::task::spawn_blocking(move || stt::transcribe_pcm(&audio_samples))
        .await
        .map_err(|e| format!("STT task execution error: {}", e))?
}

#[tauri::command]
fn cancel_recording(state: State<'_, AppState>) -> Result<(), String> {
    audio::cancel_mic_recording(&state)
}

#[tauri::command]
async fn get_ollama_models() -> Result<Vec<String>, String> {
    ollama::fetch_models().await
}

#[tauri::command]
async fn pull_ollama_model(model_name: String) -> Result<String, String> {
    ollama::pull_model(&model_name).await
}

#[tauri::command]
async fn send_ollama_chat(
    messages: Vec<ChatMessage>,
    model: String,
    system_prompt: String,
    temperature: f32,
) -> Result<String, String> {
    ollama::chat_completion(&messages, &model, &system_prompt, temperature).await
}

#[tauri::command]
fn get_tts_voices(state: State<'_, AppState>) -> Result<Vec<VoiceDto>, String> {
    tts::list_voices(&state)
}

#[tauri::command]
fn speak_text(
    state: State<'_, AppState>,
    text: String,
    voice_id: Option<String>,
    speed: f32,
    volume: f32,
    use_edge: bool,
) -> Result<(), String> {
    tts::speak(&state, &text, voice_id, speed, volume, use_edge)
}

#[tauri::command]
fn stop_speaking(state: State<'_, AppState>) -> Result<(), String> {
    tts::stop_all_speech(&state)
}

#[tauri::command]
fn get_chat_history(state: State<'_, AppState>) -> Result<Vec<ChatMessage>, String> {
    if let Ok(lock) = state.history.lock() {
        if !lock.is_empty() {
            return Ok(lock.clone());
        }
    }
    let loaded = history::load_history_from_disk();
    if let Ok(mut lock) = state.history.lock() {
        *lock = loaded.clone();
    }
    Ok(loaded)
}

#[tauri::command]
fn save_chat_history(
    state: State<'_, AppState>,
    messages: Vec<ChatMessage>,
) -> Result<(), String> {
    if let Ok(mut lock) = state.history.lock() {
        *lock = messages.clone();
    }
    history::save_history_to_disk(&messages)
}

#[tauri::command]
fn clear_chat_history(state: State<'_, AppState>) -> Result<(), String> {
    if let Ok(mut lock) = state.history.lock() {
        lock.clear();
    }
    history::clear_history_on_disk()
}

#[tauri::command]
fn export_chat_history(messages: Vec<ChatMessage>) -> Result<String, String> {
    history::export_history_as_text(&messages)
}

#[tauri::command]
async fn download_whisper_model() -> Result<String, String> {
    tokio::task::spawn_blocking(stt::download_whisper_model_sync)
        .await
        .map_err(|e| format!("Download task error: {}", e))?
}

fn main() {
    let app_state = AppState::new();

    // Preload history from disk
    let loaded = history::load_history_from_disk();
    if let Ok(mut lock) = app_state.history.lock() {
        *lock = loaded;
    }

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_microphones,
            start_recording,
            stop_recording_and_transcribe,
            cancel_recording,
            get_ollama_models,
            pull_ollama_model,
            send_ollama_chat,
            get_tts_voices,
            speak_text,
            stop_speaking,
            get_chat_history,
            save_chat_history,
            clear_chat_history,
            export_chat_history,
            download_whisper_model
        ])
        .run(tauri::generate_context!())
        .expect("error while running Jarvis Voice Chat tauri application");
}
