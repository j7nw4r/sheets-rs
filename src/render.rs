use ratatui::Frame;
use ratatui::style::Style;
use unicode_width::UnicodeWidthStr;

use crate::app::App;
use crate::cell::{CellKey, column_label, is_formula};
use crate::mode::Mode;
use crate::style as s;

/// Main render function — draws the entire UI.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let buf = frame.buffer_mut();

    let row_label_w = app.row_label_width();
    let cell_w = app.cell_width;
    let visible_rows = app.visible_rows();
    let visible_cols = app.visible_cols();

    // Selection bounds for highlighting
    let sel_bounds = if app.mode == Mode::Select {
        Some(app.selection_bounds())
    } else {
        None
    };

    // ---- Column headers ----
    let header_y = 0u16;

    // Row label corner
    let corner = format!("{:>width$}", "", width = row_label_w);
    set_string(buf, 0, header_y, &corner, s::header());

    // Column headers
    for ci in 0..visible_cols {
        let col = app.col_offset + ci;
        if col >= app.col_count {
            break;
        }
        let label = column_label(col);
        let x = (row_label_w + 1 + ci * (cell_w + 1)) as u16;
        let padded = fit_center(&label, cell_w);

        let style = if col == app.selected_col {
            s::selected_cell()
        } else {
            s::header()
        };
        set_string(buf, x, header_y, &padded, style);
    }

    // ---- Top border ----
    let border_y = 1u16;
    let mut border_line = String::new();
    border_line.push_str(&" ".repeat(row_label_w));
    border_line.push('┌');
    for ci in 0..visible_cols {
        if ci > 0 {
            border_line.push('┬');
        }
        border_line.push_str(&"─".repeat(cell_w));
    }
    border_line.push('┐');
    set_string(buf, 0, border_y, &border_line, s::border());

    // ---- Grid rows ----
    for ri in 0..visible_rows {
        let row = app.row_offset + ri;
        if row >= app.row_count {
            break;
        }
        let y = (2 + ri) as u16;
        if y >= area.height.saturating_sub(2) {
            break;
        }

        // Row label
        let row_label = format!("{:>width$}", row + 1, width = row_label_w);
        let row_label_style = if row == app.selected_row {
            s::selected_cell()
        } else {
            s::header()
        };
        set_string(buf, 0, y, &row_label, row_label_style);

        // Cells
        for ci in 0..visible_cols {
            let col = app.col_offset + ci;
            if col >= app.col_count {
                break;
            }
            let x = (row_label_w + 1 + ci * (cell_w + 1)) as u16;

            // Border separator
            let sep_x = (row_label_w + ci * (cell_w + 1)) as u16;
            set_string(buf, sep_x, y, "│", s::border());

            let is_selected = row == app.selected_row && col == app.selected_col;
            let is_in_selection = sel_bounds
                .as_ref()
                .map(|b| b.contains(CellKey::new(row, col)))
                .unwrap_or(false);

            // Get display value
            let display = if is_selected && app.mode == Mode::Insert {
                // Show the editing buffer
                format_editing(&app.editing_value, app.editing_cursor, cell_w)
            } else {
                let val = app.display_value(row, col);
                fit_right(&val, cell_w)
            };

            // Determine style
            let raw = app.cell_value(row, col);
            let style = if is_selected {
                s::selected_cell()
            } else if is_in_selection {
                s::selection()
            } else if is_formula(raw) {
                let display_val = app.display_value(row, col);
                if display_val.starts_with('#') {
                    s::error()
                } else {
                    s::formula()
                }
            } else {
                s::cell()
            };

            set_string(buf, x, y, &display, style);
        }

        // Right border
        let right_x = (row_label_w + visible_cols * (cell_w + 1)) as u16;
        if right_x < area.width {
            set_string(buf, right_x, y, "│", s::border());
        }
    }

    // ---- Bottom border ----
    let bottom_y = (2 + visible_rows.min(app.row_count - app.row_offset)) as u16;
    if bottom_y < area.height.saturating_sub(2) {
        let mut border_line = String::new();
        border_line.push_str(&" ".repeat(row_label_w));
        border_line.push('└');
        for ci in 0..visible_cols {
            if ci > 0 {
                border_line.push('┴');
            }
            border_line.push_str(&"─".repeat(cell_w));
        }
        border_line.push('┘');
        set_string(buf, 0, bottom_y, &border_line, s::border());
    }

    // ---- Status bar ----
    let status_y = area.height.saturating_sub(2);
    if status_y > 0 {
        // Clear the status line
        let blank = " ".repeat(area.width as usize);
        set_string(buf, 0, status_y, &blank, s::status_bar());

        // Mode indicator
        let mode_str = format!(" {} ", app.mode);
        set_string(buf, 0, status_y, &mode_str, s::mode_indicator());

        // Cell reference + raw value
        let cell_info = format!(
            " {} = {} ",
            app.current_cell_ref(),
            app.cell_value(app.selected_row, app.selected_col)
        );
        let info_x = mode_str.width() as u16 + 1;
        set_string(buf, info_x, status_y, &cell_info, s::status_bar());

        // File path on the right
        if let Some(ref path) = app.file_path {
            let path_str = format!(" {path} ");
            let path_x = area.width.saturating_sub(path_str.width() as u16);
            set_string(buf, path_x, status_y, &path_str, s::status_bar());
        }
    }

    // ---- Command line / message ----
    let cmd_y = area.height.saturating_sub(1);
    if cmd_y > 0 {
        let blank = " ".repeat(area.width as usize);
        set_string(buf, 0, cmd_y, &blank, Style::default());

        if app.mode == Mode::Command {
            let prefix = app.prompt_kind.prefix();
            let prompt = format!("{}{}", prefix, app.command_buffer);
            set_string(buf, 0, cmd_y, &prompt, Style::default());
        } else if !app.command_message.is_empty() {
            let style = if app.command_error {
                s::error()
            } else {
                Style::default()
            };
            set_string(buf, 0, cmd_y, &app.command_message, style);
        }
    }
}

/// Set a string in the buffer at (x, y) with a style.
fn set_string(buf: &mut ratatui::buffer::Buffer, x: u16, y: u16, s: &str, style: Style) {
    let area = *buf.area();
    if y >= area.height || x >= area.width {
        return;
    }
    let max_w = (area.width - x) as usize;
    let mut col = x;
    for ch in s.chars() {
        if (col - x) as usize >= max_w {
            break;
        }
        buf[(col, y)].set_char(ch).set_style(style);
        col += 1;
    }
}

/// Fit a string into a fixed width, right-aligned (for cell values).
fn fit_right(s: &str, width: usize) -> String {
    let w = UnicodeWidthStr::width(s);
    if w > width {
        // Truncate
        let mut result = String::new();
        let mut current_w = 0;
        for ch in s.chars() {
            let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if current_w + cw > width {
                break;
            }
            result.push(ch);
            current_w += cw;
        }
        result
    } else {
        format!("{:>width$}", s, width = width)
    }
}

/// Fit a string into a fixed width, center-aligned (for headers).
fn fit_center(s: &str, width: usize) -> String {
    let w = UnicodeWidthStr::width(s);
    if w >= width {
        s[..width.min(s.len())].to_string()
    } else {
        let left_pad = (width - w) / 2;
        let right_pad = width - w - left_pad;
        format!(
            "{}{}{}",
            " ".repeat(left_pad),
            s,
            " ".repeat(right_pad)
        )
    }
}

/// Format the editing buffer with cursor indicator.
fn format_editing(value: &str, _cursor: usize, width: usize) -> String {
    // Show the value with cursor position indicated
    let display = if value.is_empty() {
        " ".repeat(width)
    } else {
        fit_right(value, width)
    };
    display
}
