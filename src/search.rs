use crate::app::App;

/// Search forward from the current position.
pub fn search_next(app: &mut App) {
    if app.search_query.is_empty() {
        return;
    }
    let direction = app.search_direction;
    search_in_direction(app, direction);
}

/// Search backward from the current position.
pub fn search_prev(app: &mut App) {
    if app.search_query.is_empty() {
        return;
    }
    let direction = -app.search_direction;
    search_in_direction(app, direction);
}

/// Search in a given direction (1 = forward, -1 = backward).
fn search_in_direction(app: &mut App, direction: i8) {
    let query = app.search_query.to_lowercase();
    let total_cells = app.row_count * app.col_count;

    if total_cells == 0 {
        return;
    }

    let start = app.selected_row * app.col_count + app.selected_col;

    for step in 1..=total_cells {
        let idx = if direction >= 0 {
            (start + step) % total_cells
        } else {
            (start + total_cells - step) % total_cells
        };

        let row = idx / app.col_count;
        let col = idx % app.col_count;

        let display = app.display_value(row, col);
        let raw = app.cell_value(row, col);

        if display.to_lowercase().contains(&query)
            || raw.to_lowercase().contains(&query)
        {
            app.push_jump();
            app.selected_row = row;
            app.selected_col = col;
            app.ensure_visible();
            app.command_message = format!("/{}", app.search_query);
            app.command_error = false;
            return;
        }
    }

    app.command_message = format!("Pattern not found: {}", app.search_query);
    app.command_error = true;
}
