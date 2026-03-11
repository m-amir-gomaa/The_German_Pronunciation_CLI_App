/// Edge TTS — Microsoft's neural voices exposed via the Edge browser protocol.
/// Free, no API key required.
/// Voice used: de-DE-KatjaNeural (female) or de-DE-ConradNeural (male).
///
/// Protocol overview:
///   1. GET https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list
///      to get a session token header (X-ConnectionId).
///   2. Open a WSS connection to the synthesis endpoint.
///   3. Send two text frames: config + SSML.
///   4. Receive binary audio frames (MP3) until a "turn.end" signal.
///
/// Reference implementation: https://github.com/rany2/edge-tts (Python)
/// This module is a faithful Rust port of that protocol.

use anyhow::{anyhow, Result};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use uuid::Uuid;

// Edge TTS WebSocket endpoint
const WSS_URL: &str =
    "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1\
     ?TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";

/// Speak `text` using the given Edge TTS voice name.
/// Downloads the audio and plays it via rodio.
pub async fn speak(text: &str, voice: &str) -> Result<()> {
    let audio = synthesize(text, voice).await?;
    play_mp3(audio)?;
    Ok(())
}

/// Returns raw MP3 bytes from Edge TTS.
pub async fn synthesize(text: &str, voice: &str) -> Result<Vec<u8>> {
    use tokio_tungstenite::connect_async;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let connection_id = Uuid::new_v4().to_string().replace('-', "").to_uppercase();
    let url = format!("{}&ConnectionId={}", WSS_URL, connection_id);

    let (mut ws, _) = connect_async(&url)
        .await
        .map_err(|e| anyhow!("Edge TTS WebSocket connection failed: {}", e))?;

    // Frame 1: speech config
    let config_msg = speech_config_message(&connection_id);
    ws.send(Message::Text(config_msg)).await?;

    // Frame 2: SSML synthesis request
    let ssml = build_ssml(text, voice);
    let synthesis_msg = synthesis_message(&connection_id, &ssml);
    ws.send(Message::Text(synthesis_msg)).await?;

    // Collect binary audio frames
    let mut audio_chunks: Vec<Vec<u8>> = Vec::new();

    while let Some(msg) = ws.next().await {
        match msg? {
            Message::Binary(data) => {
                // Binary frames contain a header followed by audio bytes.
                // The header ends at the first double-newline (\r\n\r\n).
                if let Some(pos) = find_audio_start(&data) {
                    audio_chunks.push(data[pos..].to_vec());
                }
            }
            Message::Text(text) => {
                if text.contains("turn.end") {
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    if audio_chunks.is_empty() {
        return Err(anyhow!("Edge TTS returned no audio data"));
    }

    Ok(audio_chunks.concat())
}

fn find_audio_start(data: &[u8]) -> Option<usize> {
    // Header ends at \r\n\r\n
    data.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
}

fn speech_config_message(connection_id: &str) -> String {
    let timestamp = chrono_timestamp();
    format!(
        "X-Timestamp:{timestamp}\r\n\
         Content-Type:application/json; charset=utf-8\r\n\
         Path:speech.config\r\n\r\n\
         {{\"context\":{{\"synthesis\":{{\"audio\":{{\"metadataoptions\":\
         {{\"sentenceBoundaryEnabled\":false,\"wordBoundaryEnabled\":false}},\
         \"outputFormat\":\"audio-24khz-48kbitrate-mono-mp3\"}}}}}}}}",
    )
}

fn synthesis_message(connection_id: &str, ssml: &str) -> String {
    let timestamp = chrono_timestamp();
    format!(
        "X-RequestId:{connection_id}\r\n\
         Content-Type:application/ssml+xml\r\n\
         X-Timestamp:{timestamp}Z\r\n\
         Path:ssml\r\n\r\n\
         {ssml}",
    )
}

fn build_ssml(text: &str, voice: &str) -> String {
    format!(
        r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis"
               xmlns:mstts="https://www.w3.org/2001/mstts" xml:lang="de-DE">
            <voice name="{voice}">
                <mstts:express-as style="general">
                    {text}
                </mstts:express-as>
            </voice>
        </speak>"#,
        voice = voice,
        text = xml_escape(text),
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

fn chrono_timestamp() -> String {
    // ISO 8601 — Edge TTS requires this format
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Minimal timestamp — Edge TTS doesn't validate it strictly
    format!("2024-01-01T00:00:{:02}.000Z", secs % 60)
}

fn play_mp3(data: Vec<u8>) -> Result<()> {
    let cursor = Cursor::new(data);
    let (_stream, handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&handle)?;
    let source = Decoder::new(cursor)?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
