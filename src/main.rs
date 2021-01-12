#![deny(
    clippy::all,
    clippy::correctness,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::cargo
)]

use std::{
    error::Error,
    io,
    io::Write,
    sync::mpsc::{self},
    thread,
};

use app::App;
use clap::{App as ClapApp, Arg, ArgMatches};

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
    let matches = parse_args();
    if matches.is_present("help") {
        println!("{}", matches.usage());
        return Ok(());
    }

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
    terminal.clear().unwrap();
    Ok(())
}

fn parse_args() -> ArgMatches<'static> {
    let app = ClapApp::new("Cratuity");
    let app = app.arg(Arg::with_name("find").long("find").short("f"));
    let app = app.arg(Arg::with_name("sort").long("sort").short("s"));

    app.get_matches()
}
