use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

/// Handle a key event in visual selection mode.
pub fn update(app: &mut App, key: KeyEvent) {
    let count = app.get_count();

    match (key.code, key.modifiers) {
        // -- Exit selection --
        (KeyCode::Esc, _) | (KeyCode::Char('v'), KeyModifiers::NONE) => {
            app.exit_select();
            app.clear_count();
        }

        // -- Navigation (extends selection) --
        (KeyCode::Char('j') | KeyCode::Down, _) => {
            app.move_down(count);
            app.clear_count();
        }
        (KeyCode::Char('k') | KeyCode::Up, _) => {
            app.move_up(count);
            app.clear_count();
        }
        (KeyCode::Char('h') | KeyCode::Left, _) => {
            app.move_left(count);
            app.clear_count();
        }
        (KeyCode::Char('l') | KeyCode::Right, _) => {
            app.move_right(count);
            app.clear_count();
        }

        // -- Selection operations --
        (KeyCode::Char('y'), KeyModifiers::NONE) => {
            crate::clipboard::yank_selection(app);
            app.exit_select();
        }
        (KeyCode::Char('Y'), KeyModifiers::SHIFT) => {
            crate::clipboard::yank_selection_refs(app);
            app.exit_select();
        }
        (KeyCode::Char('x'), KeyModifiers::NONE) => {
            crate::clipboard::cut_selection(app);
            app.exit_select();
        }
        (KeyCode::Char('d'), KeyModifiers::NONE) => {
            crate::clipboard::cut_selection(app);
            app.exit_select();
        }

        // -- Count buffer --
        (KeyCode::Char(c), KeyModifiers::NONE) if c.is_ascii_digit() => {
            app.count_buffer.push(c);
        }

        _ => {}
    }
}
