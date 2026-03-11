/// IPA generation via espeak-ng.
/// espeak-ng is the gold standard for free German IPA transcription.
///
/// Install: sudo apt install espeak-ng
/// Test:    espeak-ng -v de -q --ipa "Entschuldigung"

use anyhow::{anyhow, Context, Result};
use std::process::Command;

/// Returns the IPA transcription of a German word or phrase.
pub fn get_ipa(text: &str) -> Result<String> {
    let output = Command::new("espeak-ng")
        .args([
            "-v", "de",    // German voice
            "-q",          // quiet (no audio)
            "--ipa",       // output IPA
            text,
        ])
        .output()
        .context("Failed to run espeak-ng — is it installed? (sudo apt install espeak-ng)")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("espeak-ng failed: {}", stderr));
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    // espeak-ng outputs leading/trailing whitespace and sometimes a stress mark prefix
    let ipa = raw.trim().to_string();

    if ipa.is_empty() {
        return Err(anyhow!("espeak-ng returned empty IPA for: {}", text));
    }

    Ok(ipa)
}
