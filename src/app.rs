use std::collections::HashMap;

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};

use crate::cell::{CellKey, CellRange, cell_ref, is_formula};
use crate::mode::{Mode, PendingAction, PromptKind};

/// Default cell display width in characters.
pub const DEFAULT_CELL_WIDTH: usize = 12;
/// Minimum number of rows in a new spreadsheet.
pub const MIN_ROWS: usize = 100;
/// Minimum number of columns.
pub const MIN_COLS: usize = 26;

/// Clipboard contents for copy/paste operations.
#[derive(Debug, Clone)]
pub struct Clipboard {
    /// 2D grid of cell values (rows x cols).
    pub cells: Vec<Vec<String>>,
    /// Source row of the top-left cell.
    pub source_row: usize,
    /// Source col of the top-left cell.
    pub source_col: usize,
    /// Whether this was a reference copy (Y) vs value copy (y).
    pub is_reference: bool,
}

/// Snapshot of state for undo/redo.
#[derive(Debug, Clone)]
pub struct UndoState {
    pub cells: HashMap<CellKey, String>,
    pub row_count: usize,
    pub col_count: usize,
    pub selected_row: usize,
    pub selected_col: usize,
}

/// The main application state — the single source of truth.
pub struct App {
    // -- Grid dimensions --
    pub row_count: usize,
    pub col_count: usize,

    // -- Cell data (sparse) --
    pub cells: HashMap<CellKey, String>,

    // -- Viewport --
    pub row_offset: usize,
    pub col_offset: usize,
    pub cell_width: usize,

    // -- Selection --
    pub selected_row: usize,
    pub selected_col: usize,

    // -- Visual selection anchor --
    pub select_row: usize,
    pub select_col: usize,
    pub select_rows: bool,

    // -- Mode state --
    pub mode: Mode,
    pub prompt_kind: PromptKind,
    pub pending: PendingAction,

    // -- Command/prompt buffer --
    pub command_buffer: String,
    pub command_cursor: usize,
    pub command_message: String,
    pub command_error: bool,

    // -- Count buffer for repeat commands --
    pub count_buffer: String,

    // -- Register --
    pub active_register: Option<char>,

    // -- File --
    pub file_path: Option<String>,
    pub delimiter: u8,

    // -- Search --
    pub search_query: String,
    pub search_direction: i8, // 1 = forward, -1 = backward

    // -- Clipboard & registers --
    pub clipboard: Option<Clipboard>,
    pub registers: HashMap<char, Clipboard>,

    // -- Marks --
    pub marks: HashMap<char, CellKey>,

    // -- Jump history --
    pub jump_back: Vec<CellKey>,
    pub jump_forward: Vec<CellKey>,

    // -- Undo/Redo --
    pub undo_stack: Vec<UndoState>,
    pub redo_stack: Vec<UndoState>,

    // -- Insert mode editing --
    pub editing_value: String,
    pub editing_cursor: usize,

    // -- Macro/repeat --
    pub insert_keys: Vec<KeyEvent>,
    pub recording_insert: bool,
    pub last_change: Vec<KeyEvent>,
    pub replaying_change: bool,

    // -- Terminal size --
    pub term_width: u16,
    pub term_height: u16,

    // -- Quit flag --
    pub should_quit: bool,
}

impl App {
    /// Create a new empty spreadsheet.
    pub fn new() -> Self {
        Self {
            row_count: MIN_ROWS,
            col_count: MIN_COLS,
            cells: HashMap::new(),
            row_offset: 0,
            col_offset: 0,
            cell_width: DEFAULT_CELL_WIDTH,
            selected_row: 0,
            selected_col: 0,
            select_row: 0,
            select_col: 0,
            select_rows: false,
            mode: Mode::Normal,
            prompt_kind: PromptKind::None,
            pending: PendingAction::None,
            command_buffer: String::new(),
            command_cursor: 0,
            command_message: String::new(),
            command_error: false,
            count_buffer: String::new(),
            active_register: None,
            file_path: None,
            delimiter: b',',
            search_query: String::new(),
            search_direction: 1,
            clipboard: None,
            registers: HashMap::new(),
            marks: HashMap::new(),
            jump_back: Vec::new(),
            jump_forward: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            editing_value: String::new(),
            editing_cursor: 0,
            insert_keys: Vec::new(),
            recording_insert: false,
            last_change: Vec::new(),
            replaying_change: false,
            term_width: 80,
            term_height: 24,
            should_quit: false,
        }
    }

    // ---- Cell access ----

    /// Get the raw value of a cell (the stored string, may be a formula).
    pub fn cell_value(&self, row: usize, col: usize) -> &str {
        self.cells
            .get(&CellKey::new(row, col))
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Get the display value of a cell (formulas are evaluated).
    pub fn display_value(&self, row: usize, col: usize) -> String {
        let raw = self.cell_value(row, col);
        if is_formula(raw) {
            self.evaluate_formula(row, col)
        } else {
            raw.to_string()
        }
    }

    /// Set a cell's value. Empty string removes the cell from the map.
    pub fn set_cell_value(&mut self, row: usize, col: usize, value: String) {
        let key = CellKey::new(row, col);
        if value.is_empty() {
            self.cells.remove(&key);
        } else {
            self.cells.insert(key, value);
        }
        // Expand grid if needed
        if row >= self.row_count {
            self.row_count = row + 1;
        }
        if col >= self.col_count {
            self.col_count = col + 1;
        }
    }

    /// Evaluate a formula cell — delegates to formula module.
    /// Returns the display string (number, error, etc.)
    pub fn evaluate_formula(&self, row: usize, col: usize) -> String {
        crate::formula::evaluate_cell(self, row, col)
    }

    // ---- Computed properties ----

    /// The width of row labels (number of digits in max row + padding).
    pub fn row_label_width(&self) -> usize {
        let digits = format!("{}", self.row_count).len();
        digits + 2 // padding
    }

    /// Number of visible columns that fit in the terminal width.
    pub fn visible_cols(&self) -> usize {
        let available = self.term_width as usize - self.row_label_width();
        // Each column takes cell_width + 1 (for border)
        (available / (self.cell_width + 1)).max(1)
    }

    /// Number of visible rows that fit in the terminal height.
    /// Subtract: 1 header row + 1 top border + 1 status bar + 1 command line = 4
    pub fn visible_rows(&self) -> usize {
        (self.term_height as usize).saturating_sub(4).max(1)
    }

    /// Get the current cell reference string (e.g. "A1").
    pub fn current_cell_ref(&self) -> String {
        cell_ref(CellKey::new(self.selected_row, self.selected_col))
    }

    /// Get the count from the count buffer, defaulting to 1 if empty.
    pub fn get_count(&self) -> usize {
        self.count_buffer.parse().unwrap_or(1)
    }

    /// Clear the count buffer.
    pub fn clear_count(&mut self) {
        self.count_buffer.clear();
    }

    /// Get the selection bounds (normalized so start <= end).
    pub fn selection_bounds(&self) -> CellRange {
        if self.select_rows {
            CellRange::new(
                CellKey::new(self.selected_row.min(self.select_row), 0),
                CellKey::new(
                    self.selected_row.max(self.select_row),
                    self.col_count.saturating_sub(1),
                ),
            )
        } else {
            CellRange::new(
                CellKey::new(self.selected_row, self.selected_col),
                CellKey::new(self.select_row, self.select_col),
            )
        }
    }

    // ---- Undo/Redo ----

    /// Push current state onto the undo stack.
    pub fn push_undo(&mut self) {
        self.undo_stack.push(UndoState {
            cells: self.cells.clone(),
            row_count: self.row_count,
            col_count: self.col_count,
            selected_row: self.selected_row,
            selected_col: self.selected_col,
        });
        self.redo_stack.clear();
    }

    /// Undo the last change.
    pub fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(UndoState {
                cells: self.cells.clone(),
                row_count: self.row_count,
                col_count: self.col_count,
                selected_row: self.selected_row,
                selected_col: self.selected_col,
            });
            self.cells = state.cells;
            self.row_count = state.row_count;
            self.col_count = state.col_count;
            self.selected_row = state.selected_row;
            self.selected_col = state.selected_col;
        }
    }

    /// Redo the last undone change.
    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(UndoState {
                cells: self.cells.clone(),
                row_count: self.row_count,
                col_count: self.col_count,
                selected_row: self.selected_row,
                selected_col: self.selected_col,
            });
            self.cells = state.cells;
            self.row_count = state.row_count;
            self.col_count = state.col_count;
            self.selected_row = state.selected_row;
            self.selected_col = state.selected_col;
        }
    }

    // ---- Mode transitions ----

    /// Enter insert mode, loading the current cell value into the editor.
    pub fn enter_insert(&mut self) {
        let val = self.cell_value(self.selected_row, self.selected_col).to_string();
        self.editing_value = val;
        self.editing_cursor = self.editing_value.len();
        self.mode = Mode::Insert;
        self.recording_insert = true;
        self.insert_keys.clear();
        self.push_undo();
    }

    /// Enter insert mode with cursor at the beginning.
    pub fn enter_insert_start(&mut self) {
        self.enter_insert();
        self.editing_cursor = 0;
    }

    /// Enter insert mode with the cell cleared.
    pub fn enter_insert_clear(&mut self) {
        self.push_undo();
        self.editing_value.clear();
        self.editing_cursor = 0;
        self.mode = Mode::Insert;
        self.recording_insert = true;
        self.insert_keys.clear();
    }

    /// Enter insert mode with cursor at end (append).
    pub fn enter_insert_append(&mut self) {
        self.enter_insert();
        // cursor already at end
    }

    /// Commit the current editing value and return to normal mode.
    pub fn commit_edit(&mut self) {
        let value = self.editing_value.clone();
        self.set_cell_value(self.selected_row, self.selected_col, value);
        self.mode = Mode::Normal;
        if self.recording_insert {
            self.last_change = self.insert_keys.clone();
            self.recording_insert = false;
        }
    }

    /// Cancel editing and return to normal mode.
    pub fn cancel_edit(&mut self) {
        self.mode = Mode::Normal;
        self.editing_value.clear();
        self.editing_cursor = 0;
        self.recording_insert = false;
        // Undo the push we did on enter
        self.undo();
    }

    /// Enter visual selection mode.
    pub fn enter_select(&mut self) {
        self.mode = Mode::Select;
        self.select_row = self.selected_row;
        self.select_col = self.selected_col;
        self.select_rows = false;
    }

    /// Enter row-wise visual selection mode.
    pub fn enter_select_rows(&mut self) {
        self.mode = Mode::Select;
        self.select_row = self.selected_row;
        self.select_col = self.selected_col;
        self.select_rows = true;
    }

    /// Exit visual selection mode.
    pub fn exit_select(&mut self) {
        self.mode = Mode::Normal;
    }

    /// Enter command mode.
    pub fn enter_command(&mut self) {
        self.mode = Mode::Command;
        self.prompt_kind = PromptKind::Command;
        self.command_buffer.clear();
        self.command_cursor = 0;
    }

    /// Enter search forward mode.
    pub fn enter_search_forward(&mut self) {
        self.mode = Mode::Command;
        self.prompt_kind = PromptKind::SearchForward;
        self.command_buffer.clear();
        self.command_cursor = 0;
    }

    /// Enter search backward mode.
    pub fn enter_search_backward(&mut self) {
        self.mode = Mode::Command;
        self.prompt_kind = PromptKind::SearchBackward;
        self.command_buffer.clear();
        self.command_cursor = 0;
    }

    /// Exit command/search mode.
    pub fn exit_command(&mut self) {
        self.mode = Mode::Normal;
        self.prompt_kind = PromptKind::None;
        self.command_buffer.clear();
        self.command_cursor = 0;
    }

    // ---- Jump history ----

    /// Record current position in jump-back history.
    pub fn push_jump(&mut self) {
        self.jump_back.push(CellKey::new(self.selected_row, self.selected_col));
        self.jump_forward.clear();
    }

    /// Jump back to previous position.
    pub fn jump_back(&mut self) {
        if let Some(pos) = self.jump_back.pop() {
            self.jump_forward
                .push(CellKey::new(self.selected_row, self.selected_col));
            self.selected_row = pos.row;
            self.selected_col = pos.col;
        }
    }

    /// Jump forward to next position.
    pub fn jump_forward(&mut self) {
        if let Some(pos) = self.jump_forward.pop() {
            self.jump_back
                .push(CellKey::new(self.selected_row, self.selected_col));
            self.selected_row = pos.row;
            self.selected_col = pos.col;
        }
    }

    // ---- Resize ----

    pub fn resize(&mut self, width: u16, height: u16) {
        self.term_width = width;
        self.term_height = height;
    }

    // ---- Main update dispatch ----

    /// Handle a key event, dispatching to the current mode handler.
    pub fn update(&mut self, key: KeyEvent) {
        // Record keys for macro replay when in insert mode
        if self.mode == Mode::Insert && self.recording_insert {
            self.insert_keys.push(key);
        }

        match self.mode {
            Mode::Normal => self.update_normal(key),
            Mode::Insert => self.update_insert(key),
            Mode::Select => self.update_select(key),
            Mode::Command => self.update_command(key),
        }
    }

    /// Handle a mouse event.
    pub fn update_mouse(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::Down(_) => {
                let col = (event.column as usize)
                    .saturating_sub(self.row_label_width() + 1);
                let row = (event.row as usize).saturating_sub(2); // header + top border
                let target_col = self.col_offset + col / (self.cell_width + 1);
                let target_row = self.row_offset + row;
                if target_row < self.row_count && target_col < self.col_count {
                    self.selected_row = target_row;
                    self.selected_col = target_col;
                }
            }
            MouseEventKind::ScrollDown => {
                self.move_down(3);
            }
            MouseEventKind::ScrollUp => {
                self.move_up(3);
            }
            _ => {}
        }
    }

    // ---- Navigation helpers (implemented in navigate.rs) ----

    pub fn move_up(&mut self, count: usize) {
        crate::navigate::move_up(self, count);
    }

    pub fn move_down(&mut self, count: usize) {
        crate::navigate::move_down(self, count);
    }

    pub fn move_left(&mut self, count: usize) {
        crate::navigate::move_left(self, count);
    }

    pub fn move_right(&mut self, count: usize) {
        crate::navigate::move_right(self, count);
    }

    pub fn ensure_visible(&mut self) {
        crate::navigate::ensure_visible(self);
    }

    // ---- Mode-specific update (implemented in their modules) ----

    fn update_normal(&mut self, key: KeyEvent) {
        crate::normal_mode::update(self, key);
    }

    fn update_insert(&mut self, key: KeyEvent) {
        crate::insert_mode::update(self, key);
    }

    fn update_select(&mut self, key: KeyEvent) {
        crate::select_mode::update(self, key);
    }

    fn update_command(&mut self, key: KeyEvent) {
        crate::command::update(self, key);
    }
}
