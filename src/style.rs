use ratatui::style::{Color, Modifier, Style};

// Grid colors
pub const BORDER_COLOR: Color = Color::DarkGray;
pub const HEADER_BG: Color = Color::DarkGray;
pub const HEADER_FG: Color = Color::White;
pub const SELECTED_BG: Color = Color::Blue;
pub const SELECTED_FG: Color = Color::White;
pub const SELECTION_BG: Color = Color::Rgb(60, 60, 100);
pub const SELECTION_FG: Color = Color::White;
pub const FORMULA_FG: Color = Color::Green;
pub const ERROR_FG: Color = Color::Red;
pub const STATUS_BG: Color = Color::DarkGray;
pub const STATUS_FG: Color = Color::White;

/// Style for column/row headers
pub fn header() -> Style {
    Style::default().fg(HEADER_FG).bg(HEADER_BG)
}

/// Style for the currently selected cell
pub fn selected_cell() -> Style {
    Style::default().fg(SELECTED_FG).bg(SELECTED_BG)
}

/// Style for cells within a visual selection
pub fn selection() -> Style {
    Style::default().fg(SELECTION_FG).bg(SELECTION_BG)
}

/// Style for normal (unselected) cells
pub fn cell() -> Style {
    Style::default()
}

/// Style for formula display values
pub fn formula() -> Style {
    Style::default().fg(FORMULA_FG)
}

/// Style for error display values
pub fn error() -> Style {
    Style::default().fg(ERROR_FG)
}

/// Style for the status bar
pub fn status_bar() -> Style {
    Style::default().fg(STATUS_FG).bg(STATUS_BG)
}

/// Style for the mode indicator in the status bar
pub fn mode_indicator() -> Style {
    Style::default()
        .fg(STATUS_FG)
        .bg(STATUS_BG)
        .add_modifier(Modifier::BOLD)
}

/// Style for grid borders
pub fn border() -> Style {
    Style::default().fg(BORDER_COLOR)
}
