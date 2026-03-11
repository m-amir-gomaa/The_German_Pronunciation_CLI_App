use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::{stdout, Write};

use crate::config::Config;
use crate::db::Db;
use crate::input::mode::{Buffer, Mode};
use crate::input::{normal, insert, visual};
use crate::autocomplete::Autocomplete;

pub async fn run(config: &Config, db: &Db) -> Result<()> {
    let initial_mode = if config.editor.vim_mode
        && config.editor.vim_default_mode == "normal"
    {
        Mode::Normal
    } else {
        Mode::Insert
    };

    let mut buf = Buffer::new(initial_mode);
    let ac = Autocomplete::new(config.ui.autocomplete_limit);
    let mut ac_results: Vec<String> = Vec::new();
    let mut ac_index: Option<usize> = None;

    enable_raw_mode()?;
    let mut stdout = stdout();

    loop {
        // ── render prompt ────────────────────────────────────────────────────
        execute!(
            stdout,
            cursor::MoveToColumn(0),
            Clear(ClearType::CurrentLine),
        )?;

        let mode_label = match &buf.mode {
            Mode::Normal           => "NOR",
            Mode::Insert           => "INS",
            Mode::Visual { .. }    => "VIS",
        };
        let mode_color = match &buf.mode {
            Mode::Normal           => Color::Blue,
            Mode::Insert           => Color::Green,
            Mode::Visual { .. }    => Color::Magenta,
        };

        execute!(
            stdout,
            SetForegroundColor(mode_color),
            Print(format!("[{}] ", mode_label)),
            ResetColor,
            Print("❯ "),
            Print(buf.as_string()),
        )?;
        // Move cursor to correct column
        execute!(
            stdout,
            cursor::MoveToColumn((buf.cursor + 6) as u16),
        )?;
        stdout.flush()?;

        // ── render autocomplete suggestions ──────────────────────────────────
        if !ac_results.is_empty() {
            execute!(stdout, Print("\r\n"))?;
            for (i, word) in ac_results.iter().enumerate() {
                let selected = ac_index.map_or(false, |idx| idx == i);
                if selected {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        Print(format!("  ▶ {}\r\n", word)),
                        ResetColor,
                    )?;
                } else {
                    execute!(stdout, Print(format!("    {}\r\n", word)))?;
                }
            }
            // Move back up
            let lines = ac_results.len() + 1;
            execute!(stdout, cursor::MoveUp(lines as u16))?;
        }

        // ── read next key event ──────────────────────────────────────────────
        let ev = event::read()?;
        let key = match ev {
            Event::Key(k) => k,
            _ => continue,
        };

        // ── handle Tab cycling through autocomplete ──────────────────────────
        if key.code == KeyCode::Tab && !ac_results.is_empty() {
            ac_index = Some(match ac_index {
                None       => 0,
                Some(i)    => (i + 1) % ac_results.len(),
            });
            continue;
        }

        // If Enter is pressed while an autocomplete suggestion is highlighted, confirm it
        if key.code == KeyCode::Enter {
            if let Some(idx) = ac_index {
                if let Some(word) = ac_results.get(idx) {
                    buf.clear();
                    for c in word.chars() { buf.insert_char(c); }
                    ac_results.clear();
                    ac_index = None;
                    continue;
                }
            }
        }

        // ── dispatch to current mode handler ─────────────────────────────────
        let submit;
        let quit;
        let autocomplete;

        match buf.mode.clone() {
            Mode::Normal => {
                use normal::NormalAction::*;
                let action = normal::handle(&mut buf, key);
                submit      = matches!(action, Submit);
                quit        = matches!(action, Quit);
                autocomplete = false;
            }
            Mode::Insert => {
                use insert::InsertAction::*;
                let action = insert::handle(&mut buf, key);
                submit       = matches!(action, Submit);
                quit         = matches!(action, Quit);
                autocomplete = matches!(action, Autocomplete);
            }
            Mode::Visual { anchor } => {
                use visual::VisualAction::*;
                let action = visual::handle(&mut buf, key, anchor);
                submit       = false;
                quit         = matches!(action, Quit);
                autocomplete = false;
            }
        }

        if quit {
            break;
        }

        if autocomplete {
            let query = buf.as_string();
            ac_results = ac.search(&query);
            ac_index = if ac_results.is_empty() { None } else { Some(0) };
            continue;
        }

        // Clear autocomplete when user types normally
        if !ac_results.is_empty() && !autocomplete {
            ac_results.clear();
            ac_index = None;
        }

        if submit {
            let word = buf.as_string().trim().to_string();
            if word.is_empty() { continue; }

            execute!(stdout, Print("\r\n"))?;
            disable_raw_mode()?;

            // Run the pronounce flow
            let ipa = crate::ipa::get_ipa(&word)?;
            println!("IPA: /{}/", ipa);
            crate::tts::speak(&word, config).await?;

            enable_raw_mode()?;
            buf.clear();
        }
    }

    disable_raw_mode()?;
    execute!(stdout, Print("\r\nBye!\r\n"))?;
    Ok(())
}
