use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::input::mode::{Buffer, Mode};

pub enum VisualAction {
    Continue,
    EnterNormal,
    Quit,
}

/// Handle a key event while in Visual mode.
/// `anchor` is the position where Visual mode was entered.
pub fn handle(buf: &mut Buffer, event: KeyEvent, anchor: usize) -> VisualAction {
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        match event.code {
            KeyCode::Char('c') | KeyCode::Char('d') => return VisualAction::Quit,
            _ => return VisualAction::Continue,
        }
    }

    match event.code {
        KeyCode::Esc => {
            buf.mode = Mode::Normal;
            VisualAction::EnterNormal
        }

        // ── cursor movement (same as Normal) ────────────────────────────────
        KeyCode::Char('h') | KeyCode::Left  => { buf.move_left();         VisualAction::Continue }
        KeyCode::Char('l') | KeyCode::Right => { buf.move_right();        VisualAction::Continue }
        KeyCode::Char('w')                  => { buf.move_word_forward();  VisualAction::Continue }
        KeyCode::Char('b')                  => { buf.move_word_backward(); VisualAction::Continue }
        KeyCode::Char('0') | KeyCode::Home  => { buf.move_line_start();   VisualAction::Continue }
        KeyCode::Char('$') | KeyCode::End   => { buf.move_line_end();     VisualAction::Continue }

        // ── operators on selection ───────────────────────────────────────────
        KeyCode::Char('d') => {
            let (start, end) = selection_range(anchor, buf.cursor);
            buf.delete_range(start, end + 1);
            buf.mode = Mode::Normal;
            VisualAction::EnterNormal
        }
        KeyCode::Char('y') => {
            let (start, end) = selection_range(anchor, buf.cursor);
            buf.yank_range(start, end + 1);
            buf.mode = Mode::Normal;
            VisualAction::EnterNormal
        }
        KeyCode::Char('c') => {
            // Delete selection and switch to Insert
            let (start, end) = selection_range(anchor, buf.cursor);
            buf.delete_range(start, end + 1);
            buf.cursor = start;
            buf.mode = Mode::Insert;
            VisualAction::EnterNormal // caller checks buf.mode for actual transition
        }

        _ => VisualAction::Continue,
    }
}

/// Returns (start, end) inclusive indices regardless of direction.
fn selection_range(anchor: usize, cursor: usize) -> (usize, usize) {
    if anchor <= cursor {
        (anchor, cursor)
    } else {
        (cursor, anchor)
    }
}
