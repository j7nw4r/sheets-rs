use crate::app::App;

/// Move selection up by `count` rows.
pub fn move_up(app: &mut App, count: usize) {
    app.selected_row = app.selected_row.saturating_sub(count);
    ensure_visible(app);
}

/// Move selection down by `count` rows.
pub fn move_down(app: &mut App, count: usize) {
    app.selected_row = (app.selected_row + count).min(app.row_count.saturating_sub(1));
    ensure_visible(app);
}

/// Move selection left by `count` columns.
pub fn move_left(app: &mut App, count: usize) {
    app.selected_col = app.selected_col.saturating_sub(count);
    ensure_visible(app);
}

/// Move selection right by `count` columns.
pub fn move_right(app: &mut App, count: usize) {
    app.selected_col = (app.selected_col + count).min(app.col_count.saturating_sub(1));
    ensure_visible(app);
}

/// Scroll the viewport so the selected cell is visible.
pub fn ensure_visible(app: &mut App) {
    let visible_rows = app.visible_rows();
    let visible_cols = app.visible_cols();

    // Vertical scroll
    if app.selected_row < app.row_offset {
        app.row_offset = app.selected_row;
    } else if app.selected_row >= app.row_offset + visible_rows {
        app.row_offset = app.selected_row - visible_rows + 1;
    }

    // Horizontal scroll
    if app.selected_col < app.col_offset {
        app.col_offset = app.selected_col;
    } else if app.selected_col >= app.col_offset + visible_cols {
        app.col_offset = app.selected_col - visible_cols + 1;
    }
}

/// Move half a page down.
pub fn half_page_down(app: &mut App, count: usize) {
    let half = app.visible_rows() / 2;
    move_down(app, half * count);
}

/// Move half a page up.
pub fn half_page_up(app: &mut App, count: usize) {
    let half = app.visible_rows() / 2;
    move_up(app, half * count);
}

/// Move to top of visible window.
pub fn window_top(app: &mut App) {
    app.selected_row = app.row_offset;
}

/// Move to middle of visible window.
pub fn window_middle(app: &mut App) {
    app.selected_row = app.row_offset + app.visible_rows() / 2;
}

/// Move to bottom of visible window.
pub fn window_bottom(app: &mut App) {
    let visible = app.visible_rows();
    app.selected_row = (app.row_offset + visible).min(app.row_count).saturating_sub(1);
}

/// Move to the first row.
pub fn go_to_top(app: &mut App) {
    app.selected_row = 0;
    ensure_visible(app);
}

/// Move to the last row with data.
pub fn go_to_bottom(app: &mut App) {
    app.selected_row = app.row_count.saturating_sub(1);
    ensure_visible(app);
}

/// Move to column 0.
pub fn go_to_first_col(app: &mut App) {
    app.selected_col = 0;
    ensure_visible(app);
}

/// Move to the last column with data in the current row.
pub fn go_to_last_col(app: &mut App) {
    app.selected_col = app.col_count.saturating_sub(1);
    ensure_visible(app);
}

/// Find the first non-blank column in the current row.
pub fn first_non_blank_col(app: &mut App) {
    for col in 0..app.col_count {
        if !app.cell_value(app.selected_row, col).is_empty() {
            app.selected_col = col;
            ensure_visible(app);
            return;
        }
    }
}

/// Scroll so the selected row is at the top.
pub fn scroll_top(app: &mut App) {
    app.row_offset = app.selected_row;
}

/// Scroll so the selected row is centered.
pub fn scroll_center(app: &mut App) {
    let half = app.visible_rows() / 2;
    app.row_offset = app.selected_row.saturating_sub(half);
}

/// Scroll so the selected row is at the bottom.
pub fn scroll_bottom(app: &mut App) {
    let visible = app.visible_rows();
    app.row_offset = app.selected_row.saturating_sub(visible.saturating_sub(1));
}

/// Go to a specific cell reference string (e.g. "A1").
pub fn go_to_cell(app: &mut App, cell_ref: &str) {
    if let Some(key) = crate::cell::parse_cell_ref(cell_ref) {
        app.push_jump();
        app.selected_row = key.row.min(app.row_count.saturating_sub(1));
        app.selected_col = key.col.min(app.col_count.saturating_sub(1));
        ensure_visible(app);
    }
}
