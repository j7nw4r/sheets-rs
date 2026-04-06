use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

/// Handle a key event in insert mode.
pub fn update(app: &mut App, key: KeyEvent) {
    app.update_insert_internal(key);
}

impl App {
    /// Internal insert mode key handler (also used by dot-repeat replay).
    pub fn update_insert_internal(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            // -- Exit insert mode --
            (KeyCode::Esc, _) => {
                self.commit_edit();
            }
            (KeyCode::Enter, _) => {
                self.commit_edit();
                self.move_down(1);
            }
            (KeyCode::Tab, _) => {
                self.commit_edit();
                self.move_right(1);
            }
            (KeyCode::BackTab, _) => {
                self.commit_edit();
                self.move_left(1);
            }

            // -- Text editing --
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.editing_value.insert(self.editing_cursor, c);
                self.editing_cursor += 1;
            }
            (KeyCode::Backspace, _) => {
                if self.editing_cursor > 0 {
                    self.editing_cursor -= 1;
                    self.editing_value.remove(self.editing_cursor);
                }
            }
            (KeyCode::Delete, _) => {
                if self.editing_cursor < self.editing_value.len() {
                    self.editing_value.remove(self.editing_cursor);
                }
            }

            // -- Cursor movement --
            (KeyCode::Left, _) | (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                self.editing_cursor = self.editing_cursor.saturating_sub(1);
            }
            (KeyCode::Right, _) | (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                if self.editing_cursor < self.editing_value.len() {
                    self.editing_cursor += 1;
                }
            }
            (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                self.editing_cursor = 0;
            }
            (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.editing_cursor = self.editing_value.len();
            }

            // -- Kill to end of line --
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                self.editing_value.truncate(self.editing_cursor);
            }
            // -- Kill to start of line --
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.editing_value = self.editing_value[self.editing_cursor..].to_string();
                self.editing_cursor = 0;
            }
            // -- Kill previous word --
            (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                let mut pos = self.editing_cursor;
                // Skip whitespace
                while pos > 0 && self.editing_value.as_bytes().get(pos - 1) == Some(&b' ') {
                    pos -= 1;
                }
                // Skip word chars
                while pos > 0 && self.editing_value.as_bytes().get(pos - 1) != Some(&b' ') {
                    pos -= 1;
                }
                self.editing_value = format!(
                    "{}{}",
                    &self.editing_value[..pos],
                    &self.editing_value[self.editing_cursor..]
                );
                self.editing_cursor = pos;
            }

            _ => {}
        }
    }
}
