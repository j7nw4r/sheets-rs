use std::io::{self, IsTerminal, Read};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use sheets::app::App;
use sheets::cell::parse_cell_ref;
use sheets::dsv;
use sheets::render;

#[derive(Parser, Debug)]
#[command(name = "sheets", about = "A vim-inspired terminal spreadsheet editor")]
struct Cli {
    /// CSV/TSV file to open
    file: Option<String>,

    /// Cell references or assignments (e.g. B9, B7=10)
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut app = App::new();

    // Load file if provided
    if let Some(ref path) = cli.file {
        dsv::load_file(&mut app, path)?;
    } else if !io::stdin().is_terminal() {
        // Read from stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        dsv::load_string(&mut app, &input)?;
    }

    // CLI mode: query/set cells without TUI
    if !cli.args.is_empty() {
        return run_cli(&mut app, &cli.args);
    }

    // Interactive TUI mode
    run_tui(&mut app)
}

/// Handle CLI queries like `sheets file.csv B9` or `sheets file.csv B7=10`.
fn run_cli(app: &mut App, args: &[String]) -> Result<()> {
    for arg in args {
        if let Some((cell_str, value)) = arg.split_once('=') {
            // Assignment: B7=10
            if let Some(key) = parse_cell_ref(cell_str) {
                app.set_cell_value(key.row, key.col, value.to_string());
            } else {
                eprintln!("Invalid cell reference: {cell_str}");
            }
        } else {
            // Query: B9
            if let Some(key) = parse_cell_ref(arg) {
                println!("{}", app.display_value(key.row, key.col));
            } else {
                eprintln!("Invalid cell reference: {arg}");
            }
        }
    }

    // If any assignments were made, save the file
    if args.iter().any(|a| a.contains('=')) {
        if let Some(ref path) = app.file_path {
            dsv::save_file(app, path)?;
        }
    }

    Ok(())
}

/// Run the interactive TUI.
fn run_tui(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get initial terminal size
    let size = terminal.size()?;
    app.resize(size.width, size.height);

    // Main event loop
    loop {
        terminal.draw(|frame| render::draw(frame, app))?;

        match event::read()? {
            Event::Key(key) => {
                app.update(key);
            }
            Event::Mouse(mouse) => {
                app.update_mouse(mouse);
            }
            Event::Resize(w, h) => {
                app.resize(w, h);
            }
            _ => {}
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
