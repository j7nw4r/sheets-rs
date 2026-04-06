use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use sheets::app::App;
use sheets::dsv;
use sheets::mode::Mode;

// ---- Test helpers ----

fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn shift_key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn key_code(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn type_str(app: &mut App, s: &str) {
    for c in s.chars() {
        app.update(key(c));
    }
}

/// Create an app with a 3x3 grid:
///   A1=1  B1=2  C1=3
///   A2=4  B2=5  C2=6
///   A3=7  B3=8  C3=9
fn app_with_data() -> App {
    let mut app = App::new();
    app.resize(120, 40);
    dsv::load_string(&mut app, "1,2,3\n4,5,6\n7,8,9\n").unwrap();
    app
}

// ---- Navigation ----

#[test]
fn test_hjkl_movement() {
    let mut app = app_with_data();

    // Start at (0,0)
    assert_eq!(app.selected_row, 0);
    assert_eq!(app.selected_col, 0);

    // j moves down
    app.update(key('j'));
    assert_eq!(app.selected_row, 1);
    assert_eq!(app.selected_col, 0);

    // l moves right
    app.update(key('l'));
    assert_eq!(app.selected_row, 1);
    assert_eq!(app.selected_col, 1);

    // k moves up
    app.update(key('k'));
    assert_eq!(app.selected_row, 0);
    assert_eq!(app.selected_col, 1);

    // h moves left
    app.update(key('h'));
    assert_eq!(app.selected_row, 0);
    assert_eq!(app.selected_col, 0);

    // h at col 0 stays at 0
    app.update(key('h'));
    assert_eq!(app.selected_col, 0);

    // k at row 0 stays at 0
    app.update(key('k'));
    assert_eq!(app.selected_row, 0);
}

#[test]
fn test_gg_and_big_g() {
    let mut app = app_with_data();

    // Move down first
    app.update(key('j'));
    app.update(key('j'));
    assert_eq!(app.selected_row, 2);

    // gg goes to top
    app.update(key('g'));
    app.update(key('g'));
    assert_eq!(app.selected_row, 0);

    // G goes to bottom
    app.update(shift_key('G'));
    assert_eq!(app.selected_row, app.row_count - 1);
}

#[test]
fn test_zero_and_dollar() {
    let mut app = app_with_data();

    // Move right
    app.update(key('l'));
    app.update(key('l'));
    assert_eq!(app.selected_col, 2);

    // 0 goes to first column
    app.update(key('0'));
    assert_eq!(app.selected_col, 0);

    // $ goes to last column
    app.update(shift_key('$'));
    assert_eq!(app.selected_col, app.col_count - 1);
}

#[test]
fn test_count_movement() {
    let mut app = app_with_data();

    // 3j moves down 3 rows
    app.update(key('3'));
    app.update(key('j'));
    assert_eq!(app.selected_row, 3);

    // 2l moves right 2 cols
    app.update(key('2'));
    app.update(key('l'));
    assert_eq!(app.selected_col, 2);
}

#[test]
fn test_half_page_scroll() {
    let mut app = app_with_data();

    let half = app.visible_rows() / 2;

    // Ctrl-D moves down half page
    app.update(ctrl('d'));
    assert_eq!(app.selected_row, half);

    // Ctrl-U moves back up
    app.update(ctrl('u'));
    assert_eq!(app.selected_row, 0);
}

// ---- Insert mode ----

#[test]
fn test_enter_insert_and_edit() {
    let mut app = app_with_data();

    // i enters insert mode
    app.update(key('i'));
    assert_eq!(app.mode, Mode::Insert);

    // Existing value "1" is loaded
    assert_eq!(app.editing_value, "1");

    // Type additional text
    type_str(&mut app, "00");
    assert_eq!(app.editing_value, "100");

    // Esc commits
    app.update(key_code(KeyCode::Esc));
    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.cell_value(0, 0), "100");
}

#[test]
fn test_insert_clear() {
    let mut app = app_with_data();
    assert_eq!(app.cell_value(0, 0), "1");

    // c clears cell and enters insert
    app.update(key('c'));
    assert_eq!(app.mode, Mode::Insert);
    assert_eq!(app.editing_value, "");

    type_str(&mut app, "new");
    app.update(key_code(KeyCode::Esc));
    assert_eq!(app.cell_value(0, 0), "new");
}

#[test]
fn test_insert_tab_moves_right() {
    let mut app = app_with_data();

    app.update(key('c'));
    type_str(&mut app, "x");
    app.update(key_code(KeyCode::Tab));

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.cell_value(0, 0), "x");
    assert_eq!(app.selected_col, 1);
}

#[test]
fn test_insert_enter_moves_down() {
    let mut app = app_with_data();

    app.update(key('c'));
    type_str(&mut app, "x");
    app.update(key_code(KeyCode::Enter));

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.cell_value(0, 0), "x");
    assert_eq!(app.selected_row, 1);
}

#[test]
fn test_insert_escape_commits() {
    let mut app = app_with_data();

    app.update(key('i'));
    // Clear existing and type new
    app.update(ctrl('u')); // kill line
    type_str(&mut app, "hello");
    app.update(key_code(KeyCode::Esc));

    assert_eq!(app.cell_value(0, 0), "hello");
}

#[test]
fn test_insert_backspace() {
    let mut app = app_with_data();

    app.update(key('i'));
    // Value is "1", cursor at end
    type_str(&mut app, "23");
    assert_eq!(app.editing_value, "123");

    app.update(key_code(KeyCode::Backspace));
    assert_eq!(app.editing_value, "12");

    app.update(key_code(KeyCode::Esc));
    assert_eq!(app.cell_value(0, 0), "12");
}

// ---- Visual selection ----

#[test]
fn test_visual_select_yank() {
    let mut app = app_with_data();

    // v enters visual mode
    app.update(key('v'));
    assert_eq!(app.mode, Mode::Select);

    // Extend selection right and down
    app.update(key('l'));
    app.update(key('j'));

    // y yanks and exits
    app.update(key('y'));
    assert_eq!(app.mode, Mode::Normal);

    // Clipboard should have a 2x2 grid
    let clip = app.clipboard.as_ref().unwrap();
    assert_eq!(clip.cells.len(), 2);
    assert_eq!(clip.cells[0].len(), 2);
    assert_eq!(clip.cells[0][0], "1");
    assert_eq!(clip.cells[0][1], "2");
    assert_eq!(clip.cells[1][0], "4");
    assert_eq!(clip.cells[1][1], "5");
}

#[test]
fn test_visual_select_cut() {
    let mut app = app_with_data();

    app.update(key('v'));
    app.update(key('l'));
    app.update(key('x'));

    assert_eq!(app.mode, Mode::Normal);
    // Cells should be cleared
    assert_eq!(app.cell_value(0, 0), "");
    assert_eq!(app.cell_value(0, 1), "");
    // Unselected cells remain
    assert_eq!(app.cell_value(0, 2), "3");
}

// ---- Command mode ----

#[test]
fn test_command_quit() {
    let mut app = app_with_data();

    // :q should quit
    app.update(shift_key(':'));
    assert_eq!(app.mode, Mode::Command);
    type_str(&mut app, "q");
    app.update(key_code(KeyCode::Enter));
    assert!(app.should_quit);
}

#[test]
fn test_command_goto() {
    let mut app = app_with_data();

    // :goto A3 moves to row 2, col 0
    app.update(shift_key(':'));
    type_str(&mut app, "goto A3");
    app.update(key_code(KeyCode::Enter));

    assert_eq!(app.selected_row, 2);
    assert_eq!(app.selected_col, 0);
}

#[test]
fn test_command_cell_ref_shorthand() {
    let mut app = app_with_data();

    // :B2 as goto shorthand
    app.update(shift_key(':'));
    type_str(&mut app, "B2");
    app.update(key_code(KeyCode::Enter));

    assert_eq!(app.selected_row, 1);
    assert_eq!(app.selected_col, 1);
}

#[test]
fn test_command_save_load() {
    let mut app = app_with_data();
    app.set_cell_value(0, 0, "hello".into());

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.csv");
    let path_str = path.to_str().unwrap();

    // :w path
    app.update(shift_key(':'));
    type_str(&mut app, &format!("w {path_str}"));
    app.update(key_code(KeyCode::Enter));
    assert!(!app.command_error);

    // Load into a fresh app via :e
    let mut app2 = App::new();
    app2.resize(120, 40);
    app2.update(shift_key(':'));
    type_str(&mut app2, &format!("e {path_str}"));
    app2.update(key_code(KeyCode::Enter));

    assert_eq!(app2.cell_value(0, 0), "hello");
    assert_eq!(app2.cell_value(0, 1), "2");
}

// ---- Clipboard ----

#[test]
fn test_yank_paste_cell() {
    let mut app = app_with_data();

    // Yank cell A1 (value "1") — need to use visual select for single cell
    app.update(key('v'));
    app.update(key('y'));

    // Move to B2
    app.update(key('j'));
    app.update(key('l'));
    assert_eq!(app.selected_row, 1);
    assert_eq!(app.selected_col, 1);

    // Paste
    app.update(key('p'));
    assert_eq!(app.cell_value(1, 1), "1");
}

#[test]
fn test_yank_paste_row() {
    let mut app = app_with_data();

    // yy yanks row 0
    app.update(key('y'));
    app.update(key('y'));

    // Move down and paste
    app.update(key('j'));
    app.update(key('j'));
    app.update(key('j'));
    app.update(key('p'));

    // Row 3 should now have the values from row 0
    assert_eq!(app.cell_value(3, 0), "1");
    assert_eq!(app.cell_value(3, 1), "2");
    assert_eq!(app.cell_value(3, 2), "3");
}

#[test]
fn test_dd_deletes_row() {
    let mut app = app_with_data();

    // dd deletes row 0
    app.update(key('d'));
    app.update(key('d'));

    // Row 0 should now have what was row 1
    assert_eq!(app.cell_value(0, 0), "4");
    assert_eq!(app.cell_value(0, 1), "5");
    assert_eq!(app.cell_value(0, 2), "6");

    // Row 1 should have what was row 2
    assert_eq!(app.cell_value(1, 0), "7");
    assert_eq!(app.cell_value(1, 1), "8");
    assert_eq!(app.cell_value(1, 2), "9");
}

#[test]
fn test_named_register() {
    let mut app = app_with_data();

    // "a yank cell via visual select
    app.update(shift_key('"'));
    app.update(key('a'));
    app.update(key('v'));
    app.update(key('y'));

    // Move to empty cell
    app.update(key('j'));
    app.update(key('j'));
    app.update(key('j'));

    // "a paste
    app.update(shift_key('"'));
    app.update(key('a'));
    app.update(key('p'));

    assert_eq!(app.cell_value(3, 0), "1");
}

#[test]
fn test_formula_ref_rewrite_on_paste() {
    let mut app = App::new();
    app.resize(120, 40);

    // A1=10, A2=20, A3=SUM formula
    app.set_cell_value(0, 0, "10".into());
    app.set_cell_value(1, 0, "20".into());
    app.set_cell_value(2, 0, "=A1+A2".into());
    assert_eq!(app.display_value(2, 0), "30");

    // Yank A3 (the formula cell)
    app.selected_row = 2;
    app.update(key('v'));
    app.update(key('y'));

    // Paste at B3 (same row, one col right)
    app.update(key('l'));
    app.update(key('p'));

    // Formula should be rewritten: =A1+A2 -> =B1+B2
    assert_eq!(app.cell_value(2, 1), "=B1+B2");
}

// ---- Undo/redo ----

#[test]
fn test_undo_redo() {
    let mut app = app_with_data();
    assert_eq!(app.cell_value(0, 0), "1");

    // Edit cell
    app.update(key('c'));
    type_str(&mut app, "changed");
    app.update(key_code(KeyCode::Esc));
    assert_eq!(app.cell_value(0, 0), "changed");

    // Undo
    app.update(key('u'));
    assert_eq!(app.cell_value(0, 0), "1");

    // Redo
    app.update(ctrl('r'));
    assert_eq!(app.cell_value(0, 0), "changed");
}

// ---- Search ----

#[test]
fn test_search_forward() {
    let mut app = app_with_data();

    // / search for "8"
    app.update(key('/'));
    type_str(&mut app, "8");
    app.update(key_code(KeyCode::Enter));

    // Should land on B3 (row=2, col=1 where value is "8")
    assert_eq!(app.selected_row, 2);
    assert_eq!(app.selected_col, 1);
}

#[test]
fn test_search_backward() {
    let mut app = app_with_data();

    // Move to bottom-right area
    app.update(key('j'));
    app.update(key('j'));
    app.update(key('l'));
    app.update(key('l'));

    // ? search backward for "1"
    app.update(shift_key('?'));
    type_str(&mut app, "1");
    app.update(key_code(KeyCode::Enter));

    // Should find "1" at A1
    assert_eq!(app.selected_row, 0);
    assert_eq!(app.selected_col, 0);
}

#[test]
fn test_search_next_prev() {
    let mut app = App::new();
    app.resize(120, 40);
    // Place "x" in multiple cells
    app.set_cell_value(0, 0, "x".into());
    app.set_cell_value(1, 0, "y".into());
    app.set_cell_value(2, 0, "x".into());
    app.set_cell_value(3, 0, "z".into());
    app.set_cell_value(4, 0, "x".into());

    // Search for "x"
    app.update(key('/'));
    type_str(&mut app, "x");
    app.update(key_code(KeyCode::Enter));

    // First match after (0,0) should be (2,0)
    assert_eq!(app.selected_row, 2);

    // n finds next
    app.update(key('n'));
    assert_eq!(app.selected_row, 4);

    // N finds previous
    app.update(shift_key('N'));
    assert_eq!(app.selected_row, 2);
}

// ---- Marks and jumps ----

#[test]
fn test_set_and_jump_to_mark() {
    let mut app = app_with_data();

    // Move to B2
    app.update(key('j'));
    app.update(key('l'));
    assert_eq!(app.selected_row, 1);
    assert_eq!(app.selected_col, 1);

    // Set mark 'a'
    app.update(key('m'));
    app.update(key('a'));

    // Move away
    app.update(key('j'));
    app.update(key('j'));
    assert_eq!(app.selected_row, 3);

    // Jump to mark 'a'
    app.update(key('\''));
    app.update(key('a'));
    assert_eq!(app.selected_row, 1);
    assert_eq!(app.selected_col, 1);
}

#[test]
fn test_jump_history() {
    let mut app = app_with_data();

    // Navigate with goto (pushes jump)
    app.update(shift_key(':'));
    type_str(&mut app, "goto A5");
    app.update(key_code(KeyCode::Enter));
    assert_eq!(app.selected_row, 4);

    // Ctrl-O jumps back
    app.update(ctrl('o'));
    assert_eq!(app.selected_row, 0);

    // Ctrl-I jumps forward
    app.update(ctrl('i'));
    assert_eq!(app.selected_row, 4);
}

// ---- Formulas (integration) ----

#[test]
fn test_formula_updates_on_cell_edit() {
    let mut app = App::new();
    app.resize(120, 40);

    app.set_cell_value(0, 0, "10".into());
    app.set_cell_value(0, 1, "=A1*2".into());
    assert_eq!(app.display_value(0, 1), "20");

    // Edit A1 to 50
    app.update(key('c'));
    type_str(&mut app, "50");
    app.update(key_code(KeyCode::Esc));

    // B1 formula should now reflect the new value
    assert_eq!(app.display_value(0, 1), "100");
}

// ---- Dot repeat ----

#[test]
fn test_dot_repeat() {
    let mut app = app_with_data();

    // Edit A1: clear and type "X"
    app.update(key('c'));
    type_str(&mut app, "X");
    app.update(key_code(KeyCode::Esc));
    assert_eq!(app.cell_value(0, 0), "X");

    // Move down
    app.update(key('j'));

    // Dot repeat
    app.update(key('.'));
    assert_eq!(app.cell_value(1, 0), "X");
}

// ---- CSV integration ----

#[test]
fn test_load_csv_and_navigate() {
    let mut app = App::new();
    app.resize(120, 40);
    dsv::load_string(&mut app, "Name,Age,City\nAlice,30,NYC\nBob,25,LA\n").unwrap();

    assert_eq!(app.cell_value(0, 0), "Name");
    assert_eq!(app.cell_value(0, 1), "Age");
    assert_eq!(app.cell_value(1, 0), "Alice");
    assert_eq!(app.cell_value(2, 2), "LA");

    // Navigate to Bob's age
    app.update(key('j'));
    app.update(key('j'));
    app.update(key('l'));
    assert_eq!(app.selected_row, 2);
    assert_eq!(app.selected_col, 1);
    assert_eq!(app.cell_value(2, 1), "25");
}
