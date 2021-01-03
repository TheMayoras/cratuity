use std::{
    error::Error,
    io,
    io::{Stdout, Write},
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};


use crates_io::CrateSearchResponse;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as TermEvent, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use input::{InputEvent, InputMonitor};
use reqwest::blocking;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    widgets::{Block, BorderType::Thick, Borders},
    Terminal,
};
use widgets::CrateWidget;

mod crates_io;
mod input;
mod widgets;

fn main() -> Result<(), Box<dyn Error>> {
    let crates_client = blocking::Client::builder()
        .user_agent("craters-tui-searcher")
        .build()
        .unwrap();

    let req = crates_client
        .get("https://crates.io/api/v1/crates?page=1&per_page=10&q=serde")
        .build()
        .unwrap();

    let resp = crates_client
        .execute(req)
        .unwrap()
        .json::<CrateSearchResponse>()
        .unwrap();
    println!("Response: {:#?}", resp);

    let mut stdout = io::stdout();
    enable_raw_mode()?;

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || InputMonitor::new(tx).monitor());

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title("Craters (A crates.io quick search TUI)")
                .borders(Borders::ALL)
                .border_type(Thick);
            let widgets = resp.crates.iter().map(CrateWidget::from);

            let splits = Layout::default()
                .horizontal_margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(block.inner(f.size()));
            widgets.zip(splits).for_each(|(w, a)| f.render_widget(w, a));

            f.render_widget(block, size);
        })?;

        match rx.recv_timeout(Duration::from_secs(100)) {
            Ok(InputEvent::Quit) | Err(_) => break,
            _ => {}
        }
    }
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}
