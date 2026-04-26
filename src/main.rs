use std::io;

use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
mod app;
mod render;
mod runtime;
mod vmoptions;

use crate::app::App;
use crate::runtime::run_app;
use crate::vmoptions::{append_vmoptions, find_apps, resolve_options_path};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Non-interactive CLI mode: jetbrains_vmoptions --apply '<option>'
    if args.len() >= 3 && args[1] == "--apply" {
        let option = &args[2];
        let apps = find_apps();
        if apps.is_empty() {
            eprintln!("No apps found in PATH");
            std::process::exit(1);
        }
        for (name, binary) in &apps {
            match resolve_options_path(binary) {
                None => eprintln!("{name}: vmoptions not found"),
                Some(path) => match append_vmoptions(&path, option) {
                    Ok(true) => println!("{name}: updated ({})", path.display()),
                    Ok(false) => println!("{name}: already set"),
                    Err(e) => eprintln!("{name}: {e}"),
                },
            }
        }
        return Ok(());
    }

    // Optional positional arg pre-fills the editor.
    let initial = args.get(1).cloned();
    let mut app = App::new(initial);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}