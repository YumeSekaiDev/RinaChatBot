use crate::state::ChatMessage;
use chrono::Local;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub fn get_app_data_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("JarvisVoiceChat");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_history_file_path() -> PathBuf {
    get_app_data_dir().join("chat_history.json")
}

pub fn load_history_from_disk() -> Vec<ChatMessage> {
    let path = get_history_file_path();
    if !path.exists() {
        return Vec::new();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str::<Vec<ChatMessage>>(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_history_to_disk(messages: &[ChatMessage]) -> Result<(), String> {
    let path = get_history_file_path();
    let json_data = serde_json::to_string_pretty(messages)
        .map_err(|e| format!("Failed to serialize chat history: {}", e))?;

    let mut file =
        File::create(&path).map_err(|e| format!("Failed to create history file: {}", e))?;
    file.write_all(json_data.as_bytes())
        .map_err(|e| format!("Failed to write history file: {}", e))?;

    Ok(())
}

pub fn clear_history_on_disk() -> Result<(), String> {
    let path = get_history_file_path();
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
    save_history_to_disk(&[])
}

pub fn export_history_as_text(messages: &[ChatMessage]) -> Result<String, String> {
    let now = Local::now();
    let date_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let file_stamp = now.format("%Y%m%d_%H%M%S").to_string();

    let mut out = String::new();
    out.push_str("=====================================================\n");
    out.push_str("          JARVIS VOICE CHAT - TRANSCRIPT EXPORT      \n");
    out.push_str(&format!("  Exported At: {}\n", date_str));
    out.push_str(&format!("  Total Messages: {}\n", messages.len()));
    out.push_str("=====================================================\n\n");

    for msg in messages {
        let role_label = match msg.role.to_lowercase().as_str() {
            "user" => "USER",
            "assistant" => "JARVIS",
            "system" => "SYSTEM",
            _ => "UNKNOWN",
        };

        out.push_str(&format!("[{}] {}:\n", msg.timestamp, role_label));
        out.push_str(&msg.content);
        out.push_str("\n\n-----------------------------------------------------\n\n");
    }

    // Determine export save location (Downloads directory if available, else app data exports/)
    let export_dir = dirs::download_dir().unwrap_or_else(|| get_app_data_dir().join("exports"));
    let _ = fs::create_dir_all(&export_dir);

    let export_filename = format!("Jarvis_Transcript_{}.txt", file_stamp);
    let export_path = export_dir.join(&export_filename);

    let mut file = File::create(&export_path)
        .map_err(|e| format!("Failed to create export file at {}: {}", export_path.display(), e))?;

    file.write_all(out.as_bytes())
        .map_err(|e| format!("Failed to write export transcript: {}", e))?;

    Ok(export_path.to_string_lossy().to_string())
}
