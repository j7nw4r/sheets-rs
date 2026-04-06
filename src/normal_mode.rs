use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::mode::PendingAction;
use crate::navigate;

/// Handle a key event in normal mode.
pub fn update(app: &mut App, key: KeyEvent) {
    // If we have a pending action, handle it first
    if app.pending != PendingAction::None {
        handle_pending(app, key);
        return;
    }

    let count = app.get_count();

    match (key.code, key.modifiers) {
        // -- Quit --
        (KeyCode::Char('q'), KeyModifiers::NONE) => app.should_quit = true,
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.should_quit = true,

        // -- Count buffer --
        (KeyCode::Char(c), KeyModifiers::NONE) if c.is_ascii_digit() && (c != '0' || !app.count_buffer.is_empty()) => {
            app.count_buffer.push(c);
        }

        // -- Navigation --
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

        // Half-page scrolling
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            navigate::half_page_down(app, count);
            app.clear_count();
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            navigate::half_page_up(app, count);
            app.clear_count();
        }

        // Window positioning
        (KeyCode::Char('H'), KeyModifiers::SHIFT) => {
            navigate::window_top(app);
            app.clear_count();
        }
        (KeyCode::Char('M'), KeyModifiers::SHIFT) => {
            navigate::window_middle(app);
            app.clear_count();
        }
        (KeyCode::Char('L'), KeyModifiers::SHIFT) => {
            navigate::window_bottom(app);
            app.clear_count();
        }

        // First/last column
        (KeyCode::Char('0'), KeyModifiers::NONE) => {
            navigate::go_to_first_col(app);
            app.clear_count();
        }
        (KeyCode::Char('$'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            navigate::go_to_last_col(app);
            app.clear_count();
        }
        (KeyCode::Char('^'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            navigate::first_non_blank_col(app);
            app.clear_count();
        }

        // Top/bottom
        (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
            navigate::go_to_bottom(app);
            app.clear_count();
        }

        // -- Insert mode --
        (KeyCode::Char('i'), KeyModifiers::NONE) => app.enter_insert(),
        (KeyCode::Char('I'), KeyModifiers::SHIFT) => app.enter_insert_start(),
        (KeyCode::Char('a'), KeyModifiers::NONE) => app.enter_insert_append(),
        (KeyCode::Char('A'), KeyModifiers::SHIFT) => {
            // Append to end of cell value
            app.enter_insert_append();
        }
        (KeyCode::Char('c'), KeyModifiers::NONE) => app.enter_insert_clear(),
        (KeyCode::Char('o'), KeyModifiers::NONE) => {
            // Insert row below and enter insert mode
            app.push_undo();
            crate::clipboard::insert_row_below(app);
            app.move_down(1);
            app.enter_insert_clear();
        }
        (KeyCode::Char('O'), KeyModifiers::SHIFT) => {
            // Insert row above and enter insert mode
            app.push_undo();
            crate::clipboard::insert_row_above(app);
            app.enter_insert_clear();
        }
        (KeyCode::Enter, _) => {
            // Enter edit mode on the current cell
            app.enter_insert();
        }

        // -- Visual selection --
        (KeyCode::Char('v'), KeyModifiers::NONE) => app.enter_select(),
        (KeyCode::Char('V'), KeyModifiers::SHIFT) => app.enter_select_rows(),

        // -- Command mode --
        (KeyCode::Char(':'), KeyModifiers::NONE | KeyModifiers::SHIFT) => app.enter_command(),

        // -- Search --
        (KeyCode::Char('/'), KeyModifiers::NONE) => app.enter_search_forward(),
        (KeyCode::Char('?'), KeyModifiers::NONE | KeyModifiers::SHIFT) => app.enter_search_backward(),
        (KeyCode::Char('n'), KeyModifiers::NONE) => {
            crate::search::search_next(app);
        }
        (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
            crate::search::search_prev(app);
        }

        // -- Pending actions --
        (KeyCode::Char('g'), KeyModifiers::NONE) => {
            app.pending = PendingAction::Goto;
        }
        (KeyCode::Char('d'), KeyModifiers::NONE) => {
            app.pending = PendingAction::Delete;
        }
        (KeyCode::Char('y'), KeyModifiers::NONE) => {
            app.pending = PendingAction::Yank;
        }
        (KeyCode::Char('z'), KeyModifiers::NONE) => {
            app.pending = PendingAction::ZScroll;
        }
        (KeyCode::Char('"'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            app.pending = PendingAction::Register;
        }
        (KeyCode::Char('m'), KeyModifiers::NONE) => {
            app.pending = PendingAction::Mark;
        }
        (KeyCode::Char('\''), KeyModifiers::NONE) => {
            app.pending = PendingAction::MarkJump { exact: false };
        }
        (KeyCode::Char('`'), KeyModifiers::NONE) => {
            app.pending = PendingAction::MarkJump { exact: true };
        }

        // -- Clipboard --
        (KeyCode::Char('p'), KeyModifiers::NONE) => {
            crate::clipboard::paste(app);
        }
        (KeyCode::Char('x'), KeyModifiers::NONE) => {
            crate::clipboard::cut_cell(app);
        }

        // -- Undo/Redo --
        (KeyCode::Char('u'), KeyModifiers::NONE) => app.undo(),
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => app.redo(),

        // -- Dot repeat --
        (KeyCode::Char('.'), KeyModifiers::NONE) => {
            replay_last_change(app);
        }

        // -- Jump history --
        (KeyCode::Char('o'), KeyModifiers::CONTROL) => app.jump_back(),
        (KeyCode::Char('i'), KeyModifiers::CONTROL) => app.jump_forward(),

        _ => {}
    }
}

/// Handle the second key of a pending multi-key command.
fn handle_pending(app: &mut App, key: KeyEvent) {
    let pending = app.pending.clone();
    app.pending = PendingAction::None;

    match pending {
        PendingAction::Goto => handle_goto(app, key),
        PendingAction::Delete => handle_delete(app, key),
        PendingAction::Yank => handle_yank(app, key),
        PendingAction::ZScroll => handle_z_scroll(app, key),
        PendingAction::Register => handle_register(app, key),
        PendingAction::Mark => handle_mark(app, key),
        PendingAction::MarkJump { exact } => handle_mark_jump(app, key, exact),
        PendingAction::None => {}
    }
}

fn handle_goto(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('g') => {
            // gg -> go to top
            navigate::go_to_top(app);
        }
        KeyCode::Char('e') => {
            // ge -> go to bottom
            navigate::go_to_bottom(app);
        }
        _ => {}
    }
}

fn handle_delete(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('d') => {
            // dd -> delete row
            crate::clipboard::delete_row(app);
        }
        _ => {}
    }
}

fn handle_yank(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') => {
            // yy -> yank row
            crate::clipboard::yank_row(app);
        }
        _ => {}
    }
}

fn handle_z_scroll(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('t') => navigate::scroll_top(app),
        KeyCode::Char('z') => navigate::scroll_center(app),
        KeyCode::Char('b') => navigate::scroll_bottom(app),
        _ => {}
    }
}

fn handle_register(app: &mut App, key: KeyEvent) {
    if let KeyCode::Char(c) = key.code {
        app.active_register = Some(c);
    }
}

fn handle_mark(app: &mut App, key: KeyEvent) {
    if let KeyCode::Char(c) = key.code {
        use crate::cell::CellKey;
        app.marks
            .insert(c, CellKey::new(app.selected_row, app.selected_col));
    }
}

fn handle_mark_jump(app: &mut App, key: KeyEvent, _exact: bool) {
    if let KeyCode::Char(c) = key.code {
        if let Some(&pos) = app.marks.get(&c) {
            app.push_jump();
            app.selected_row = pos.row;
            app.selected_col = pos.col;
            app.ensure_visible();
        }
    }
}

/// Replay the last recorded change.
fn replay_last_change(app: &mut App) {
    if app.last_change.is_empty() {
        return;
    }
    let keys = app.last_change.clone();
    app.replaying_change = true;
    app.enter_insert_clear();
    for key in keys {
        app.update_insert_internal(key);
    }
    app.replaying_change = false;
}
