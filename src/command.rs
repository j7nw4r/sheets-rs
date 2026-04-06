use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::mode::PromptKind;

/// Handle a key event in command/search prompt mode.
pub fn update(app: &mut App, key: KeyEvent) {
    match (key.code, key.modifiers) {
        // -- Cancel --
        (KeyCode::Esc, _) => {
            app.exit_command();
        }

        // -- Execute --
        (KeyCode::Enter, _) => {
            let buffer = app.command_buffer.clone();
            let kind = app.prompt_kind;
            app.exit_command();
            execute_prompt(app, kind, &buffer);
        }

        // -- Text editing --
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            app.command_buffer.insert(app.command_cursor, c);
            app.command_cursor += 1;
        }
        (KeyCode::Backspace, _) => {
            if app.command_cursor > 0 {
                app.command_cursor -= 1;
                app.command_buffer.remove(app.command_cursor);
            } else {
                app.exit_command();
            }
        }
        (KeyCode::Delete, _) => {
            if app.command_cursor < app.command_buffer.len() {
                app.command_buffer.remove(app.command_cursor);
            }
        }

        // -- Cursor movement --
        (KeyCode::Left, _) | (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
            app.command_cursor = app.command_cursor.saturating_sub(1);
        }
        (KeyCode::Right, _) | (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
            if app.command_cursor < app.command_buffer.len() {
                app.command_cursor += 1;
            }
        }
        (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
            app.command_cursor = 0;
        }
        (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
            app.command_cursor = app.command_buffer.len();
        }

        _ => {}
    }
}

/// Execute a completed prompt.
fn execute_prompt(app: &mut App, kind: PromptKind, input: &str) {
    match kind {
        PromptKind::Command => execute_command(app, input),
        PromptKind::SearchForward => {
            app.search_query = input.to_string();
            app.search_direction = 1;
            crate::search::search_next(app);
        }
        PromptKind::SearchBackward => {
            app.search_query = input.to_string();
            app.search_direction = -1;
            crate::search::search_next(app);
        }
        PromptKind::None => {}
    }
}

/// Execute a command string.
fn execute_command(app: &mut App, cmd: &str) {
    let cmd = cmd.trim();
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let command = parts[0];
    let arg = parts.get(1).map(|s| s.trim());

    match command {
        "q" | "quit" | "exit" => {
            app.should_quit = true;
        }
        "w" | "write" => {
            let path = arg
                .map(|s| s.to_string())
                .or_else(|| app.file_path.clone());
            if let Some(path) = path {
                match crate::dsv::save_file(app, &path) {
                    Ok(()) => {
                        app.file_path = Some(path.clone());
                        app.command_message = format!("Written to {path}");
                        app.command_error = false;
                    }
                    Err(e) => {
                        app.command_message = format!("Error: {e}");
                        app.command_error = true;
                    }
                }
            } else {
                app.command_message = "No file path specified".into();
                app.command_error = true;
            }
        }
        "wq" | "x" => {
            let path = arg
                .map(|s| s.to_string())
                .or_else(|| app.file_path.clone());
            if let Some(path) = path {
                match crate::dsv::save_file(app, &path) {
                    Ok(()) => app.should_quit = true,
                    Err(e) => {
                        app.command_message = format!("Error: {e}");
                        app.command_error = true;
                    }
                }
            } else {
                app.command_message = "No file path specified".into();
                app.command_error = true;
            }
        }
        "goto" => {
            if let Some(cell) = arg {
                crate::navigate::go_to_cell(app, cell);
            }
        }
        "e" | "edit" => {
            if let Some(path) = arg {
                match crate::dsv::load_file(app, path) {
                    Ok(()) => {
                        app.command_message = format!("Loaded {path}");
                        app.command_error = false;
                    }
                    Err(e) => {
                        app.command_message = format!("Error: {e}");
                        app.command_error = true;
                    }
                }
            }
        }
        "help" => {
            app.command_message = "sheets: vim-like spreadsheet editor. :q quit, :w save, :goto A1 navigate".into();
            app.command_error = false;
        }
        _ => {
            // Try as a cell reference (goto shorthand)
            if crate::cell::parse_cell_ref(cmd).is_some() {
                crate::navigate::go_to_cell(app, cmd);
            } else {
                app.command_message = format!("Unknown command: {cmd}");
                app.command_error = true;
            }
        }
    }
}
