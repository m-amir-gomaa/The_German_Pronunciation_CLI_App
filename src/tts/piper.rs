/// Piper TTS — fast, offline, CPU-only text-to-speech.
/// Runs the `piper` binary as a subprocess with the German ONNX model.
///
/// Setup (one time):
///   1. Download piper binary from https://github.com/rhasspy/piper/releases
///   2. Download German model: de_DE-thorsten-high.onnx + de_DE-thorsten-high.onnx.json
///   3. Place the .onnx file at the path set in config.toml → tts.piper_model

use anyhow::{anyhow, Context, Result};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::process::{Command, Stdio};
use std::io::Write;

/// Speak `text` using the local Piper binary + model.
pub fn speak(text: &str, model_path: &str) -> Result<()> {
    let audio = synthesize(text, model_path)?;
    play_wav(audio)?;
    Ok(())
}

/// Synthesize `text` to raw WAV bytes using Piper.
pub fn synthesize(text: &str, model_path: &str) -> Result<Vec<u8>> {
    let model_path = shellexpand::tilde(model_path).into_owned();

    // piper reads from stdin, writes WAV to stdout
    let mut child = Command::new("piper")
        .args([
            "--model", &model_path,
            "--output_raw",   // raw PCM to stdout
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to launch piper binary — is it installed and on PATH?")?;

    // Write text to piper's stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(anyhow!("Piper exited with status {}", output.status));
    }

    if output.stdout.is_empty() {
        return Err(anyhow!("Piper produced no audio output"));
    }

    Ok(output.stdout)
}

fn play_wav(data: Vec<u8>) -> Result<()> {
    let cursor = Cursor::new(data);
    let (_stream, handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&handle)?;
    let source = Decoder::new(cursor)?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
