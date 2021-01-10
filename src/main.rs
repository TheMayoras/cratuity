use std::{error::Error, io, io::Write, thread};

use app::App;

use crossbeam_channel::unbounded;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use input::{InputMonitor, TickMonitor};

use tui::{backend::CrosstermBackend, Terminal};

const TICK_INTERVAL: u64 = 400;

mod app;
mod crates_io;
mod input;
mod widgets;

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = unbounded();
    let clone = tx.clone();
    thread::spawn(move || InputMonitor::new(clone).monitor());
    let clone = tx.clone();
    thread::spawn(move || TickMonitor::new(TICK_INTERVAL, clone).monitor());
    let mut app = App::new(tx, rx);

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

    Ok(())
}
