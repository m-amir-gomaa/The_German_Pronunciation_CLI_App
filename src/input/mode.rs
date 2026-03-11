/// Current Vim editing mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual { anchor: usize },
}

/// The shared editor buffer — a Vec<char> with a cursor position.
/// All modes read/write through this struct.
#[derive(Debug, Clone)]
pub struct Buffer {
    pub chars: Vec<char>,
    /// Byte index of the cursor (0-based, points to next insert position in Insert mode)
    pub cursor: usize,
    pub mode: Mode,
    /// Yanked text (for p / P paste in Normal mode)
    pub yank: Vec<char>,
}

impl Buffer {
    pub fn new(initial_mode: Mode) -> Self {
        Self {
            chars: Vec::new(),
            cursor: 0,
            mode: initial_mode,
            yank: Vec::new(),
        }
    }

    pub fn as_string(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn clear(&mut self) {
        self.chars.clear();
        self.cursor = 0;
    }

    // ── cursor movement helpers ──────────────────────────────────────────────

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        let max = match self.mode {
            Mode::Insert => self.chars.len(),
            _ => self.chars.len().saturating_sub(1),
        };
        if self.cursor < max {
            self.cursor += 1;
        }
    }

    pub fn move_word_forward(&mut self) {
        // Skip current word chars
        while self.cursor < self.chars.len() && !self.chars[self.cursor].is_whitespace() {
            self.cursor += 1;
        }
        // Skip whitespace
        while self.cursor < self.chars.len() && self.chars[self.cursor].is_whitespace() {
            self.cursor += 1;
        }
    }

    pub fn move_word_backward(&mut self) {
        if self.cursor == 0 { return; }
        self.cursor -= 1;
        // Skip whitespace backwards
        while self.cursor > 0 && self.chars[self.cursor].is_whitespace() {
            self.cursor -= 1;
        }
        // Skip word chars backwards
        while self.cursor > 0 && !self.chars[self.cursor - 1].is_whitespace() {
            self.cursor -= 1;
        }
    }

    pub fn move_line_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_line_end(&mut self) {
        self.cursor = self.chars.len().saturating_sub(1);
    }

    // ── edit operations ──────────────────────────────────────────────────────

    pub fn insert_char(&mut self, c: char) {
        self.chars.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn delete_char_before(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.chars.remove(self.cursor);
        }
    }

    pub fn delete_char_at(&mut self) {
        if self.cursor < self.chars.len() {
            self.chars.remove(self.cursor);
            // Clamp cursor after deletion
            if self.cursor > 0 && self.cursor >= self.chars.len() {
                self.cursor = self.chars.len().saturating_sub(1);
            }
        }
    }

    /// Yank (copy) characters in [start, end) into self.yank
    pub fn yank_range(&mut self, start: usize, end: usize) {
        let end = end.min(self.chars.len());
        let start = start.min(end);
        self.yank = self.chars[start..end].to_vec();
    }

    /// Delete characters in [start, end) and yank them
    pub fn delete_range(&mut self, start: usize, end: usize) {
        let end = end.min(self.chars.len());
        let start = start.min(end);
        self.yank = self.chars.drain(start..end).collect();
        self.cursor = start.min(self.chars.len().saturating_sub(1));
    }

    /// Paste yanked text after cursor (Normal mode `p`)
    pub fn paste_after(&mut self) {
        let insert_at = (self.cursor + 1).min(self.chars.len());
        for (i, &c) in self.yank.clone().iter().enumerate() {
            self.chars.insert(insert_at + i, c);
        }
        self.cursor = insert_at + self.yank.len().saturating_sub(1);
    }

    /// Paste yanked text before cursor (Normal mode `P`)
    pub fn paste_before(&mut self) {
        for (i, &c) in self.yank.clone().iter().enumerate() {
            self.chars.insert(self.cursor + i, c);
        }
        self.cursor += self.yank.len().saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_delete() {
        let mut buf = Buffer::new(Mode::Insert);
        buf.insert_char('h');
        buf.insert_char('i');
        assert_eq!(buf.as_string(), "hi");
        buf.delete_char_before();
        assert_eq!(buf.as_string(), "h");
    }

    #[test]
    fn word_motion() {
        let mut buf = Buffer::new(Mode::Normal);
        for c in "hello world".chars() { buf.insert_char(c); }
        buf.cursor = 0;
        buf.move_word_forward();
        assert_eq!(buf.cursor, 6);
        buf.move_word_backward();
        assert_eq!(buf.cursor, 0);
    }
}
