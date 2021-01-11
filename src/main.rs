use std::{
    error::Error,
    io,
    io::Write,
    sync::mpsc::{self},
    thread,
};

use app::App;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use input::InputMonitor;

use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod crates_io;
mod input;
mod widgets;

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || InputMonitor::new(tx).monitor());
    let mut app = App::new(rx);

    let mut stdout = io::stdout();
    enable_raw_mode()?;

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            app.draw(f);
        })?;

        app.await_input();
        if app.quit {
            break;
        }
    }
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.clear();
    Ok(())
}
