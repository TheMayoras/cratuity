use std::{
    error::Error,
    io,
    io::Write,
    sync::mpsc::{self},
    thread,
};

use app::App;
use crates_io::CrateSearcher;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use input::{InputMonitor};

use tui::{
    backend::CrosstermBackend,
    Terminal,
};


mod app;
mod crates_io;
mod input;
mod widgets;

fn main() -> Result<(), Box<dyn Error>> {
    let crates_client = CrateSearcher::new().unwrap();
    //    let req = crates_client
    //        .get("https://crates.io/api/v1/crates?page=1&per_page=10&q=serde")
    //        .build()
    //        .unwrap();

    let resp = crates_client.search("serde", 1).unwrap();
    println!("Response: {:#?}", resp);

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || InputMonitor::new(tx).monitor());
    let mut app = App::new(rx);
    app.crates = Some(resp);

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
