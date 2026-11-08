use cpal::Stream;
use rodio::{OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String, // "user", "assistant", or "system"
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceDto {
    pub id: String,
    pub name: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDto {
    pub ollama_running: bool,
    pub whisper_model_present: bool,
    pub is_recording: bool,
    pub is_speaking: bool,
}

pub struct AppState {
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,
    pub active_stream: Arc<Mutex<Option<Stream>>>,
    pub stream_sample_rate: Arc<Mutex<u32>>,
    pub stream_channels: Arc<Mutex<u16>>,
    pub is_recording: Arc<AtomicBool>,
    pub is_speaking: Arc<AtomicBool>,
    pub tts_instance: Arc<Mutex<Option<tts::Tts>>>,
    pub current_process: Arc<Mutex<Option<std::process::Child>>>,
    pub rodio_sink: Arc<Mutex<Option<Sink>>>,
    pub rodio_stream: Arc<Mutex<Option<OutputStream>>>,
    pub history: Arc<Mutex<Vec<ChatMessage>>>,
}

impl AppState {
    pub fn new() -> Self {
        // Initialize Rodio output stream and sink if possible
        let (stream_opt, sink_opt) = match OutputStream::try_default() {
            Ok((stream, stream_handle)) => {
                let sink = Sink::try_new(&stream_handle).ok();
                (Some(stream), sink)
            }
            Err(_) => (None, None),
        };

        // Try initializing TTS engine (SAPI on Windows, speech-dispatcher on Linux)
        let tts_opt = tts::Tts::default().ok();

        Self {
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            active_stream: Arc::new(Mutex::new(None)),
            stream_sample_rate: Arc::new(Mutex::new(44100)),
            stream_channels: Arc::new(Mutex::new(1)),
            is_recording: Arc::new(AtomicBool::new(false)),
            is_speaking: Arc::new(AtomicBool::new(false)),
            tts_instance: Arc::new(Mutex::new(tts_opt)),
            current_process: Arc::new(Mutex::new(None)),
            rodio_sink: Arc::new(Mutex::new(sink_opt)),
            rodio_stream: Arc::new(Mutex::new(stream_opt)),
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_recording(&self, val: bool) {
        self.is_recording.store(val, Ordering::SeqCst);
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn set_speaking(&self, val: bool) {
        self.is_speaking.store(val, Ordering::SeqCst);
    }

    pub fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::SeqCst)
    }
}
