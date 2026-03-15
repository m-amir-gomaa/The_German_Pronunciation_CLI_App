# German CLI App

**Repo:** github.com/m-amir-gomaa/The_German_Pronunciation_CLI_App
**Language:** Rust
**Status:** Starting from scratch — no Rust code written yet

---

## What it does

German pronunciation trainer. IPA display, audio playback, speech recognition, spaced repetition, fuzzy search over a large word list.

The existing repo has an AI-generated version. Ignore it. Write everything yourself.

---

## Build order (do not skip ahead)

1. Word list loading from file
2. Display word + IPA in terminal
3. Accept typed input, compare to correct IPA, show diff
4. Session scoring
5. SQLite persistence — save session results, load on next run
6. Spaced repetition weighting — wrong answers surface more often
7. Audio playback (rodio crate)
8. Fuzzy search over word list

---

## Architecture rule — critical for Android later

**Keep logic and interface strictly separated from the start.**

The core library (word loading, IPA comparison, SRS algorithm, scoring) must have no dependency on terminal I/O — no stdin reads, no stdout prints inside library functions.

The CLI binary is a thin layer that calls the library and handles all terminal interaction.

Why this matters: when you later wrap this for Android, a coding agent will write a Kotlin UI that calls your Rust core via FFI. If the logic is tangled with terminal code, that becomes very hard. If it's clean, the agent can do it without your involvement.

```
src/
  lib.rs       ← all logic lives here, no I/O
  main.rs      ← CLI layer only, calls lib.rs
```

---

## Android plan (Phase 3+)

Target: non-technical user, Google Play.
Plan: coding agent writes Kotlin wrapper calling Rust core via JNI/FFI. Kotlin-literate friend verifies before publish.
Prerequisite: lib.rs must be clean of terminal I/O (see above).
Do not think about this until the CLI is complete and you use it daily.

---

## Definition of done (CLI)

- Builds with `cargo build --release` without warnings
- Loads word list, runs drill session, saves results, loads them next run
- You use it yourself every day for German practice
- README explains what it does and how to run it
