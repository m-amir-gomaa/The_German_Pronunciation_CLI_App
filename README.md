# 🦀 German Pronunciation CLI (`gp`)

A terminal-based German pronunciation trainer with:
- **IPA transcription** via `espeak-ng`
- **Human-quality TTS** — Edge TTS (Microsoft neural, online) with Piper fallback (offline)
- **Pronunciation scoring** — speak into your mic, get a % accuracy score
- **Vim modal editing** — Normal / Insert / Visual modes (configurable)
- **Fuzzy autocomplete** — Tab over ~170k German words (nucleo-matcher)
- **Word lists & flashcards** — SQLite-backed spaced repetition

---

## Install dependencies

```bash
# espeak-ng (IPA generation)
sudo apt install espeak-ng

# Piper (offline TTS) — download binary + German model
# https://github.com/rhasspy/piper/releases
# Model: de_DE-thorsten-high.onnx from https://huggingface.co/rhasspy/piper-voices

# whisper-rs requires cmake + libclang (Phase 5)
sudo apt install cmake libclang-dev
```

---

## Build

```bash
cargo build --release
# Binary: ./target/release/gp
```

---

## Usage

```bash
# Pronounce a word (IPA + audio)
gp pronounce Entschuldigung

# Practice with mic scoring
gp practice Entschuldigung

# Save a word to your list
gp save Entschuldigung

# List saved words + scores
gp list

# Flashcard drill mode
gp drill

# Interactive REPL (Vim motions + Tab autocomplete)
gp repl
```

---

## Config

Copy `config.toml.example` to `~/.config/pronouncer/config.toml` and edit.

```bash
cp config.toml.example ~/.config/pronouncer/config.toml
```

Key options:
- `editor.vim_mode` — enable/disable Vim modal editing
- `tts.preferred_voice` — `de-DE-KatjaNeural` (female) or `de-DE-ConradNeural` (male)
- `whisper.model` — `tiny` / `base` / `small`

---

## REPL Vim Keybindings

| Key | Mode | Action |
|-----|------|--------|
| `i` | Normal | Enter Insert mode |
| `a` | Normal | Insert after cursor |
| `A` | Normal | Insert at end of line |
| `I` | Normal | Insert at start |
| `v` | Normal | Enter Visual mode |
| `Esc` | Insert/Visual | Return to Normal |
| `h/l` | Normal/Visual | Move left/right |
| `w/b` | Normal/Visual | Word forward/backward |
| `0/$` | Normal/Visual | Line start/end |
| `x` | Normal | Delete char under cursor |
| `D` | Normal | Delete to end of line |
| `Y` | Normal | Yank to end of line |
| `p/P` | Normal | Paste after/before |
| `d` | Visual | Delete selection |
| `y` | Visual | Yank selection |
| `c` | Visual | Change selection |
| `Tab` | Insert | Fuzzy autocomplete |
| `Enter` | Any | Submit word → pronounce |
| `Ctrl-C` | Any | Quit |

---

## Architecture

```
src/
├── main.rs           — clap CLI commands
├── config.rs         — config.toml loader
├── input/
│   ├── mode.rs       — Buffer + Mode enum (shared state)
│   ├── normal.rs     — Normal mode keymap
│   ├── insert.rs     — Insert mode + Tab autocomplete trigger
│   ├── visual.rs     — Visual mode + operators
│   └── repl.rs       — Main REPL loop
├── autocomplete.rs   — nucleo-matcher fuzzy search over dictionary
├── tts/
│   ├── mod.rs        — Hybrid dispatcher (online → offline fallback)
│   ├── edge.rs       — Edge TTS WebSocket client
│   └── piper.rs      — Piper subprocess wrapper
├── ipa.rs            — espeak-ng IPA generation
├── stt.rs            — Whisper STT (Phase 5)
├── scoring.rs        — Levenshtein pronunciation scoring
└── db.rs             — SQLite word list + attempt history
```

---

## TTS Note

**Edge TTS** (`de-DE-KatjaNeural`) is Microsoft's neural voice used in Edge browser — completely free, no API key. Quality is indistinguishable from a paid service.

**Piper** is the offline fallback. The `de_DE-thorsten-high` model sounds natural on CPU.

---

## Development Roadmap

| Phase | Status | Description |
|-------|--------|-------------|
| 1 | 🏗️ scaffold | TTS + IPA: `gp pronounce` works |
| 2 | 🏗️ scaffold | Raw input + Insert mode |
| 3 | 🏗️ scaffold | Full Vim modal engine |
| 4 | 🏗️ scaffold | Tab fuzzy autocomplete |
| 5 | 📋 stub | STT + pronunciation scoring |
| 6 | 📋 stub | Word lists + flashcard drill |
| 7 | — | Polish: completions, config, verbose |
