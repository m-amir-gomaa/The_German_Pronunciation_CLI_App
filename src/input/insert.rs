use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::input::mode::{Buffer, Mode};

/// Return value from Insert mode key handler.
pub enum InsertAction {
    Continue,
    /// Switch back to Normal mode
    EnterNormal,
    /// Tab was pressed — caller should run fuzzy autocomplete
    Autocomplete,
    Submit,
    Quit,
}

/// Handle a key event while in Insert mode.
pub fn handle(buf: &mut Buffer, event: KeyEvent) -> InsertAction {
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        match event.code {
            KeyCode::Char('c') | KeyCode::Char('d') => return InsertAction::Quit,
            // Ctrl-w: delete word before cursor (standard shell binding)
            KeyCode::Char('w') => {
                while buf.cursor > 0
                    && buf.chars.get(buf.cursor - 1).map_or(false, |c| c.is_whitespace())
                {
                    buf.delete_char_before();
                }
                while buf.cursor > 0
                    && buf.chars.get(buf.cursor - 1).map_or(false, |c| !c.is_whitespace())
                {
                    buf.delete_char_before();
                }
                return InsertAction::Continue;
            }
            _ => return InsertAction::Continue,
        }
    }

    match event.code {
        KeyCode::Esc => {
            // Vim convention: move cursor one left when leaving Insert mode
            if buf.cursor > 0 { buf.cursor -= 1; }
            buf.mode = Mode::Normal;
            InsertAction::EnterNormal
        }
        KeyCode::Enter      => InsertAction::Submit,
        KeyCode::Tab        => InsertAction::Autocomplete,
        KeyCode::Backspace  => { buf.delete_char_before(); InsertAction::Continue }
        KeyCode::Delete     => { buf.delete_char_at();     InsertAction::Continue }
        KeyCode::Left       => { buf.move_left();          InsertAction::Continue }
        KeyCode::Right      => { buf.move_right();         InsertAction::Continue }
        KeyCode::Home       => { buf.move_line_start();    InsertAction::Continue }
        KeyCode::End        => { buf.cursor = buf.chars.len(); InsertAction::Continue }
        KeyCode::Char(c)    => { buf.insert_char(c);      InsertAction::Continue }
        _ => InsertAction::Continue,
    }
}
