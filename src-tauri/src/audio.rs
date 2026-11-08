use crate::state::AppState;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, StreamConfig};
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub fn list_input_devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to access input audio devices: {}", e))?;

    let mut names = Vec::new();
    for dev in devices {
        if let Ok(name) = dev.name() {
            names.push(name);
        }
    }

    if names.is_empty() {
        return Err("No audio input devices (microphones) found on this system.".to_string());
    }

    Ok(names)
}

pub fn start_mic_recording(state: &AppState, device_name: Option<String>) -> Result<(), String> {
    // If already recording, stop first
    if state.is_recording() {
        let _ = cancel_mic_recording(state);
    }

    let host = cpal::default_host();
    let device = if let Some(target_name) = device_name {
        let mut devices = host
            .input_devices()
            .map_err(|e| format!("Audio devices query error: {}", e))?;
        devices.find(|d| d.name().map(|n| n == target_name).unwrap_or(false))
    } else {
        host.default_input_device()
    };

    let device = device.ok_or_else(|| {
        "No matching audio input device could be selected. Please check your microphone.".to_string()
    })?;

    let default_config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get microphone input configuration: {}", e))?;

    let sample_rate = default_config.sample_rate().0;
    let channels = default_config.channels();
    let sample_format = default_config.sample_format();
    let stream_config: StreamConfig = default_config.into();

    // Reset buffer
    {
        let mut buf = state.audio_buffer.lock().unwrap();
        buf.clear();
    }
    *state.stream_sample_rate.lock().unwrap() = sample_rate;
    *state.stream_channels.lock().unwrap() = channels;

    let buffer_clone = Arc::clone(&state.audio_buffer);
    let err_fn = |err| {
        eprintln!("Audio input stream error: {}", err);
    };

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut lock) = buffer_clone.lock() {
                    lock.extend_from_slice(data);
                }
            },
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if let Ok(mut lock) = buffer_clone.lock() {
                    for &sample in data {
                        lock.push(sample.to_float_sample());
                    }
                }
            },
            err_fn,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            &stream_config,
            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                if let Ok(mut lock) = buffer_clone.lock() {
                    for &sample in data {
                        lock.push(sample.to_float_sample());
                    }
                }
            },
            err_fn,
            None,
        ),
        fmt => return Err(format!("Unsupported audio sample format: {:?}", fmt)),
    }
    .map_err(|e| format!("Failed to build audio recording stream: {}", e))?;

    stream
        .play()
        .map_err(|e| format!("Failed to start recording audio stream: {}", e))?;

    *state.active_stream.lock().unwrap() = Some(stream);
    state.set_recording(true);

    Ok(())
}

pub fn stop_mic_recording(state: &AppState) -> Result<Vec<f32>, String> {
    state.set_recording(false);

    // Drop stream to close recording device
    {
        let mut stream_lock = state.active_stream.lock().unwrap();
        *stream_lock = None;
    }

    let raw_samples = {
        let mut buf_lock = state.audio_buffer.lock().unwrap();
        let samples = buf_lock.clone();
        buf_lock.clear();
        samples
    };

    let sample_rate = *state.stream_sample_rate.lock().unwrap();
    let channels = *state.stream_channels.lock().unwrap();

    if raw_samples.is_empty() {
        return Err("No audio was recorded. Please check your microphone.".to_string());
    }

    // Minimum 0.25 seconds of audio to avoid processing empty clicks
    let min_samples = (sample_rate as usize * channels as usize) / 4;
    if raw_samples.len() < min_samples {
        return Err("Recording was too short. Please speak again.".to_string());
    }

    // Convert multi-channel to mono and resample to 16,000 Hz
    let resampled_mono = convert_and_resample(&raw_samples, channels, sample_rate, 16000);

    Ok(resampled_mono)
}

pub fn cancel_mic_recording(state: &AppState) -> Result<(), String> {
    state.set_recording(false);
    {
        let mut stream_lock = state.active_stream.lock().unwrap();
        *stream_lock = None;
    }
    {
        let mut buf_lock = state.audio_buffer.lock().unwrap();
        buf_lock.clear();
    }
    Ok(())
}

pub fn convert_and_resample(
    buffer: &[f32],
    channels: u16,
    src_rate: u32,
    target_rate: u32,
) -> Vec<f32> {
    // 1. Channel Downmixing (Multi-channel -> Mono)
    let mono = if channels > 1 {
        let ch = channels as usize;
        let num_frames = buffer.len() / ch;
        let mut mono_vec = Vec::with_capacity(num_frames);
        for i in 0..num_frames {
            let mut sum = 0.0f32;
            for c in 0..ch {
                sum += buffer[i * ch + c];
            }
            mono_vec.push(sum / (ch as f32));
        }
        mono_vec
    } else {
        buffer.to_vec()
    };

    // 2. Resampling to target_rate (typically 16000 Hz for Whisper)
    let resampled = if src_rate == target_rate {
        mono
    } else {
        let ratio = src_rate as f64 / target_rate as f64;
        let target_len = ((mono.len() as f64) / ratio).floor() as usize;
        let mut out = Vec::with_capacity(target_len);

        for i in 0..target_len {
            let src_idx = (i as f64) * ratio;
            let idx0 = src_idx.floor() as usize;
            let frac = (src_idx - idx0 as f64) as f32;

            if idx0 + 1 < mono.len() {
                let s0 = mono[idx0];
                let s1 = mono[idx0 + 1];
                out.push(s0 * (1.0 - frac) + s1 * frac);
            } else if idx0 < mono.len() {
                out.push(mono[idx0]);
            } else {
                out.push(0.0);
            }
        }
        out
    };

    // 3. Normalize peak amplitude to ensure clear input for Whisper
    let max_abs = resampled.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
    if max_abs > 0.001 {
        let boost = (0.95 / max_abs).min(5.0); // Boost quiet mics up to 5x, clamp to 0.95 peak
        resampled.into_iter().map(|s| (s * boost).clamp(-1.0, 1.0)).collect()
    } else {
        resampled
    }
}
