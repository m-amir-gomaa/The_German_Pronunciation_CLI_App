pub mod edge;
pub mod piper;

use anyhow::Result;
use crate::config::Config;

/// Speak `text` using the best available TTS engine.
/// Tries Edge TTS (online, neural quality) first.
/// Falls back to Piper (offline, local model) if network is unavailable.
pub async fn speak(text: &str, config: &Config) -> Result<()> {
    match edge::speak(text, &config.tts.preferred_voice).await {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::warn!("Edge TTS failed ({}), falling back to Piper", e);
            piper::speak(text, &config.tts.piper_model)
        }
    }
}
