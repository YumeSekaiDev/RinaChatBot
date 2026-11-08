use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

// Default to ggml-tiny.en.bin (75MB) for ultra-fast, lightweight laptop inference
const WHISPER_TINY_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin";
const WHISPER_TINY_FILENAME: &str = "ggml-tiny.en.bin";
const WHISPER_BASE_FILENAME: &str = "ggml-base.en.bin";

pub fn get_whisper_models_dir() -> PathBuf {
    let base_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("JarvisVoiceChat")
        .join("models");
    let _ = fs::create_dir_all(&base_dir);
    base_dir
}

/// Returns the active model path (tiny if present, or base if present, else default tiny path)
pub fn get_whisper_model_path() -> PathBuf {
    let dir = get_whisper_models_dir();
    let tiny_path = dir.join(WHISPER_TINY_FILENAME);
    let base_path = dir.join(WHISPER_BASE_FILENAME);

    if tiny_path.exists() {
        tiny_path
    } else if base_path.exists() {
        base_path
    } else {
        tiny_path
    }
}

pub fn is_whisper_model_downloaded() -> bool {
    let path = get_whisper_model_path();
    if path.exists() {
        if let Ok(meta) = fs::metadata(&path) {
            // tiny is ~75MB, base is ~140MB; ensure not broken 0-byte file
            return meta.len() > 10_000_000;
        }
    }
    false
}

pub fn download_whisper_model_sync() -> Result<String, String> {
    let target_path = get_whisper_model_path();
    if is_whisper_model_downloaded() {
        return Ok(format!(
            "Model is already available at: {}",
            target_path.display()
        ));
    }

    let parent = get_whisper_models_dir();
    let _ = fs::create_dir_all(&parent);

    let temp_path = parent.join(format!("{}.part", WHISPER_TINY_FILENAME));
    if temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }

    println!(
        "Downloading lightweight Whisper STT model (ggml-tiny.en.bin) from {} ...",
        WHISPER_TINY_URL
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut response = client
        .get(WHISPER_TINY_URL)
        .send()
        .map_err(|e| format!("Failed to connect to HuggingFace to download Whisper model: {}. Please check your internet connection.", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download Whisper model: HTTP {}",
            response.status()
        ));
    }

    let mut file = File::create(&temp_path)
        .map_err(|e| format!("Failed to create temporary file for Whisper model: {}", e))?;

    response
        .copy_to(&mut file)
        .map_err(|e| format!("Failed while saving Whisper model bytes: {}", e))?;

    file.flush()
        .map_err(|e| format!("Failed to flush Whisper model to disk: {}", e))?;
    drop(file);

    let final_dest = parent.join(WHISPER_TINY_FILENAME);
    fs::rename(&temp_path, &final_dest)
        .map_err(|e| format!("Failed to rename downloaded model to final path: {}", e))?;

    println!("Whisper STT model downloaded successfully to {}", final_dest.display());
    Ok(format!(
        "Whisper STT model downloaded successfully to: {}",
        final_dest.display()
    ))
}

pub fn transcribe_pcm(audio_16k_mono: &[f32]) -> Result<String, String> {
    let model_path = get_whisper_model_path();

    // Auto-download lightweight model on first run if missing
    if !is_whisper_model_downloaded() {
        download_whisper_model_sync()?;
    }

    let model_path_str = model_path
        .to_str()
        .ok_or_else(|| "Invalid model file path encoding".to_string())?;

    // Load Whisper context
    let ctx = WhisperContext::new_with_params(model_path_str, WhisperContextParameters::default())
        .map_err(|e| format!("Failed to load Whisper context: {:?}", e))?;

    let mut state = ctx
        .create_state()
        .map_err(|e| format!("Failed to create Whisper execution state: {:?}", e))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    // Use 4 threads (or hardware threads) for fast laptop inference
    params.set_n_threads(4);
    params.set_language(Some("en"));
    params.set_translate(false);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    state
        .full(params, audio_16k_mono)
        .map_err(|e| format!("Whisper model inference failed: {:?}", e))?;

    let num_segments = state.full_n_segments();

    let mut text = String::new();
    for i in 0..num_segments {
        if let Some(segment) = state.get_segment(i) {
            if let Ok(s) = segment.to_str() {
                text.push_str(s);
                text.push(' ');
            }
        }
    }

    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        Ok("(Silence or unintelligible speech)".to_string())
    } else {
        Ok(trimmed)
    }
}
