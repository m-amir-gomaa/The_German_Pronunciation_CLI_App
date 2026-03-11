/// Speech-to-Text via Whisper (local, no API required).
/// Uses whisper-rs which wraps whisper.cpp.
///
/// Phase 5 — this module is stubbed for MVP.
/// To activate:
///   1. Uncomment whisper-rs in Cargo.toml
///   2. Install build deps: sudo apt install cmake libclang-dev
///   3. Download model: base or tiny from https://huggingface.co/ggerganov/whisper.cpp
///   4. Set config.toml → whisper.model
///
/// The recording + transcription flow:
///   1. Record N seconds from default mic via cpal into a Vec<f32> buffer
///   2. Pass buffer to whisper-rs with language="de" forced
///   3. Return the transcribed string

use anyhow::Result;
use crate::config::Config;

/// Record from mic and return German transcription.
/// Stub — replace body with real whisper-rs + cpal code in Phase 5.
pub fn record_and_transcribe(config: &Config) -> Result<String> {
    // TODO Phase 5:
    //
    // use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    // use whisper_rs::{WhisperContext, FullParams, SamplingStrategy};
    //
    // 1. Open default input device with cpal
    // 2. Record 4–6 seconds of f32 mono audio at 16kHz (Whisper's required sample rate)
    // 3. Load whisper model from config.whisper.model path
    // 4. Run whisper with language = "de"
    // 5. Return the transcribed text

    tracing::warn!("STT not yet implemented — returning placeholder");
    Ok("[STT not yet implemented]".to_string())
}
