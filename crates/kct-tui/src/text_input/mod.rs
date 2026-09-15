//! A single-line text input with no interior mutability.
//!
//! `apply` consumes the input and returns a new one, which keeps the whole UI update path
//! free of `&mut` and makes every editing rule trivially testable.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInput {
    value: String,
    /// Cursor position in *characters*, not bytes — usernames and group names are routinely
    /// non-ASCII, and a byte cursor would split them.
    cursor: usize,
}

impl TextInput {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.chars().count();
        Self { value, cursor }
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    #[must_use]
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    #[must_use]
    pub fn cleared(self) -> Self {
        Self::default()
    }

    /// Handle one key. Unknown keys leave the input untouched, so a caller can pass every
    /// key through and act on the ones it cares about afterwards.
    #[must_use]
    pub fn apply(self, key: KeyEvent) -> Self {
        let control = key.modifiers.contains(KeyModifiers::CONTROL);

        match (key.code, control) {
            (KeyCode::Char(c), false) => self.insert(c),
            (KeyCode::Backspace, false) => self.delete_back(),
            (KeyCode::Delete, _) => self.delete_forward(),
            (KeyCode::Left, false) => self.move_by(-1),
            (KeyCode::Right, false) => self.move_by(1),
            (KeyCode::Home, _) | (KeyCode::Char('a'), true) => Self { cursor: 0, ..self },
            (KeyCode::End, _) | (KeyCode::Char('e'), true) => {
                let cursor = self.value.chars().count();
                Self { cursor, ..self }
            }
            // Readline habits: ^W kills the previous word, ^U the whole line.
            (KeyCode::Char('w') | KeyCode::Backspace, true) => self.delete_word_back(),
            (KeyCode::Char('u'), true) => Self::default(),
            (KeyCode::Char('k'), true) => self.truncate_at_cursor(),
            _ => self,
        }
    }

    #[must_use]
    pub fn insert(self, c: char) -> Self {
        let index = self.byte_index(self.cursor);
        let mut value = self.value;
        value.insert(index, c);
        Self {
            value,
            cursor: self.cursor + 1,
        }
    }

    #[must_use]
    fn delete_back(self) -> Self {
        if self.cursor == 0 {
            return self;
        }
        let start = self.byte_index(self.cursor - 1);
        let end = self.byte_index(self.cursor);
        let mut value = self.value;
        value.replace_range(start..end, "");
        Self {
            value,
            cursor: self.cursor - 1,
        }
    }

    #[must_use]
    fn delete_forward(self) -> Self {
        let total = self.value.chars().count();
        if self.cursor >= total {
            return self;
        }
        let start = self.byte_index(self.cursor);
        let end = self.byte_index(self.cursor + 1);
        let mut value = self.value;
        value.replace_range(start..end, "");
        Self { value, ..self }
    }

    #[must_use]
    fn delete_word_back(self) -> Self {
        let chars: Vec<char> = self.value.chars().collect();
        let skipped_spaces = (0..self.cursor)
            .rev()
            .take_while(|i| chars[*i].is_whitespace())
            .count();
        let word_start = (0..self.cursor - skipped_spaces)
            .rev()
            .take_while(|i| !chars[*i].is_whitespace())
            .count();
        let new_cursor = self.cursor - skipped_spaces - word_start;

        let value: String = chars[..new_cursor]
            .iter()
            .chain(chars[self.cursor..].iter())
            .collect();

        Self {
            value,
            cursor: new_cursor,
        }
    }

    #[must_use]
    fn truncate_at_cursor(self) -> Self {
        let index = self.byte_index(self.cursor);
        let mut value = self.value;
        value.truncate(index);
        Self { value, ..self }
    }

    #[must_use]
    fn move_by(self, delta: isize) -> Self {
        let total = self.value.chars().count();
        let cursor = self.cursor.saturating_add_signed(delta).min(total);
        Self { cursor, ..self }
    }

    fn byte_index(&self, char_index: usize) -> usize {
        self.value
            .char_indices()
            .nth(char_index)
            .map_or(self.value.len(), |(index, _)| index)
    }
}

#[cfg(test)]
mod tests;
