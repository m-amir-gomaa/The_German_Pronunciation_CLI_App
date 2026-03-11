use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::input::mode::{Buffer, Mode};

/// Return value from Normal mode key handler.
pub enum NormalAction {
    /// Stay in Normal mode, buffer may have changed
    Continue,
    /// Switch to Insert mode
    EnterInsert,
    /// Switch to Insert mode, appending after cursor
    EnterInsertAppend,
    /// Switch to Visual char mode
    EnterVisual,
    /// The user pressed Enter — submit the current buffer
    Submit,
    /// Ctrl-C / Ctrl-D — quit
    Quit,
}

/// Handle a key event while in Normal mode.
/// Mutates `buf` and returns what the REPL should do next.
pub fn handle(buf: &mut Buffer, event: KeyEvent) -> NormalAction {
    // Ctrl combos first
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        match event.code {
            KeyCode::Char('c') | KeyCode::Char('d') => return NormalAction::Quit,
            _ => return NormalAction::Continue,
        }
    }

    match event.code {
        // ── mode transitions ────────────────────────────────────────────────
        KeyCode::Char('i') => {
            buf.mode = Mode::Insert;
            NormalAction::EnterInsert
        }
        KeyCode::Char('a') => {
            buf.move_right();
            buf.mode = Mode::Insert;
            NormalAction::EnterInsertAppend
        }
        KeyCode::Char('A') => {
            buf.cursor = buf.chars.len();
            buf.mode = Mode::Insert;
            NormalAction::EnterInsertAppend
        }
        KeyCode::Char('I') => {
            buf.cursor = 0;
            buf.mode = Mode::Insert;
            NormalAction::EnterInsert
        }
        KeyCode::Char('v') => {
            buf.mode = Mode::Visual { anchor: buf.cursor };
            NormalAction::EnterVisual
        }
        KeyCode::Enter => NormalAction::Submit,

        // ── cursor movement ─────────────────────────────────────────────────
        KeyCode::Char('h') | KeyCode::Left  => { buf.move_left();          NormalAction::Continue }
        KeyCode::Char('l') | KeyCode::Right => { buf.move_right();         NormalAction::Continue }
        KeyCode::Char('w')                  => { buf.move_word_forward();  NormalAction::Continue }
        KeyCode::Char('b')                  => { buf.move_word_backward(); NormalAction::Continue }
        KeyCode::Char('0') | KeyCode::Home  => { buf.move_line_start();    NormalAction::Continue }
        KeyCode::Char('$') | KeyCode::End   => { buf.move_line_end();      NormalAction::Continue }

        // ── edit operations ─────────────────────────────────────────────────
        KeyCode::Char('x') => {
            buf.delete_char_at();
            NormalAction::Continue
        }
        KeyCode::Char('p') => {
            buf.paste_after();
            NormalAction::Continue
        }
        KeyCode::Char('P') => {
            buf.paste_before();
            NormalAction::Continue
        }

        // dd — delete whole line
        // Handled as a two-key sequence: first 'd' sets a pending state.
        // For MVP simplicity, 'D' deletes from cursor to end.
        KeyCode::Char('D') => {
            let end = buf.chars.len();
            buf.delete_range(buf.cursor, end);
            NormalAction::Continue
        }

        // yy — yank whole line
        // For MVP, 'Y' yanks from cursor to end.
        KeyCode::Char('Y') => {
            let start = buf.cursor;
            let end = buf.chars.len();
            buf.yank_range(start, end);
            NormalAction::Continue
        }

        _ => NormalAction::Continue,
    }
}
