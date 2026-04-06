use crate::app::{App, Clipboard};
use crate::cell::{CellKey, CellRange, cell_ref, is_formula, rewrite_formula_refs};

/// Copy the current cell to the clipboard.
pub fn yank_cell(app: &mut App) {
    let val = app.cell_value(app.selected_row, app.selected_col).to_string();
    let clip = Clipboard {
        cells: vec![vec![val]],
        source_row: app.selected_row,
        source_col: app.selected_col,
        is_reference: false,
    };
    store_clipboard(app, clip);
}

/// Copy the current row to the clipboard.
pub fn yank_row(app: &mut App) {
    let mut row_vals = Vec::new();
    for col in 0..app.col_count {
        row_vals.push(app.cell_value(app.selected_row, col).to_string());
    }
    let clip = Clipboard {
        cells: vec![row_vals],
        source_row: app.selected_row,
        source_col: 0,
        is_reference: false,
    };
    store_clipboard(app, clip);
}

/// Copy the visual selection to the clipboard.
pub fn yank_selection(app: &mut App) {
    let bounds = app.selection_bounds();
    let clip = extract_range(app, &bounds, false);
    store_clipboard(app, clip);
}

/// Copy cell references (not values) from the visual selection.
pub fn yank_selection_refs(app: &mut App) {
    let bounds = app.selection_bounds();
    let clip = extract_range(app, &bounds, true);
    store_clipboard(app, clip);
}

/// Cut the current cell.
pub fn cut_cell(app: &mut App) {
    app.push_undo();
    yank_cell(app);
    app.set_cell_value(app.selected_row, app.selected_col, String::new());
}

/// Cut the visual selection.
pub fn cut_selection(app: &mut App) {
    app.push_undo();
    let bounds = app.selection_bounds();
    let clip = extract_range(app, &bounds, false);
    store_clipboard(app, clip);

    // Clear the selection
    for key in bounds.iter() {
        app.set_cell_value(key.row, key.col, String::new());
    }
}

/// Delete the current row (cut it).
pub fn delete_row(app: &mut App) {
    app.push_undo();
    yank_row(app);
    shift_rows_up(app, app.selected_row);
}

/// Paste from the clipboard at the current position.
pub fn paste(app: &mut App) {
    let clip = get_clipboard(app);
    if let Some(clip) = clip {
        app.push_undo();
        let row_offset = app.selected_row as isize - clip.source_row as isize;
        let col_offset = app.selected_col as isize - clip.source_col as isize;

        for (r, row) in clip.cells.iter().enumerate() {
            for (c, val) in row.iter().enumerate() {
                let target_row = app.selected_row + r;
                let target_col = app.selected_col + c;

                let final_val = if is_formula(val) && !clip.is_reference {
                    rewrite_formula_refs(val, row_offset, col_offset)
                } else {
                    val.clone()
                };

                app.set_cell_value(target_row, target_col, final_val);
            }
        }
    }
}

/// Insert a blank row below the current row.
pub fn insert_row_below(app: &mut App) {
    shift_rows_down(app, app.selected_row + 1);
}

/// Insert a blank row above the current row.
pub fn insert_row_above(app: &mut App) {
    shift_rows_down(app, app.selected_row);
}

// ---- Internal helpers ----

fn extract_range(app: &App, bounds: &CellRange, as_reference: bool) -> Clipboard {
    let mut cells = Vec::new();
    for row in bounds.start.row..=bounds.end.row {
        let mut row_vals = Vec::new();
        for col in bounds.start.col..=bounds.end.col {
            if as_reference {
                row_vals.push(cell_ref(CellKey::new(row, col)));
            } else {
                row_vals.push(app.cell_value(row, col).to_string());
            }
        }
        cells.push(row_vals);
    }
    Clipboard {
        cells,
        source_row: bounds.start.row,
        source_col: bounds.start.col,
        is_reference: as_reference,
    }
}

fn store_clipboard(app: &mut App, clip: Clipboard) {
    // Store in active register if set, otherwise unnamed
    if let Some(reg) = app.active_register.take() {
        app.registers.insert(reg, clip);
    } else {
        app.clipboard = Some(clip);
    }
}

fn get_clipboard(app: &App) -> Option<Clipboard> {
    if let Some(reg) = app.active_register {
        app.registers.get(&reg).cloned()
    } else {
        app.clipboard.clone()
    }
}

/// Shift all rows from `from_row` down by one.
fn shift_rows_down(app: &mut App, from_row: usize) {
    // Work from the bottom up to avoid overwriting
    for row in (from_row..app.row_count).rev() {
        for col in 0..app.col_count {
            let val = app.cell_value(row, col).to_string();
            app.set_cell_value(row + 1, col, val);
        }
    }
    // Clear the inserted row
    for col in 0..app.col_count {
        app.set_cell_value(from_row, col, String::new());
    }
    app.row_count += 1;
}

/// Shift all rows from `from_row` up by one (deleting from_row).
fn shift_rows_up(app: &mut App, from_row: usize) {
    for row in from_row..app.row_count.saturating_sub(1) {
        for col in 0..app.col_count {
            let val = app.cell_value(row + 1, col).to_string();
            app.set_cell_value(row, col, val);
        }
    }
    // Clear the last row
    if app.row_count > 0 {
        let last = app.row_count - 1;
        for col in 0..app.col_count {
            app.set_cell_value(last, col, String::new());
        }
    }
}
