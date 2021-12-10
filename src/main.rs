use std::{
    error::Error,
    io,
    io::Write,
    sync::mpsc::{self},
    thread,
};

use app::App;

use crates_io::{CrateSearch, CrateSearchResponse, CrateSearcher, CratesSort};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use input::InputMonitor;

use comfy_table::{ContentArrangement, Row, Table, ToRow};
use structopt::StructOpt;

use tui::{backend::CrosstermBackend, Terminal};
use widgets::STR_FORMAT;

mod app;
mod crates_io;
mod input;
mod toast;
mod widgets;

const TABLE_STYLE: &'static str = "││ ─├─┼┤│─┼├┤ ┴  └┘";

pub(crate) fn ceil_div(a: u32, b: u32) -> u32 {
    if b == 0 {
        panic!("attempt to divide by zero");
    } else if a == 0 {
        0
    } else {
        (a + b - 1) / b
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Cratuity", about = "A simple TUI for searching Crates.io")]
/// A TUI for searching crates.io in the terminal.  
///
/// Alternatively, the find option may be used to bypass the TUI and output the
/// results directly to the terminal.
pub struct AppArgs {
    #[structopt(short, long)]
    pub find: Option<String>,

    #[structopt(short, long, default_value)]
    pub sort: CratesSort,

    #[structopt(short, long, default_value = "5")]
    pub count: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = AppArgs::clap().get_matches();
    if matches.is_present("help") {
        println!("{}", matches.usage());
        return Ok(());
    }

    let args: AppArgs = AppArgs::from_clap(&matches);
    if let Some(find) = args.find {
        cli_search(find.as_str(), args.sort, args.count)?;

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
    // terminal.clear().unwrap();
    Ok(())
}

fn cli_search(term: &str, sort: CratesSort, count: usize) -> Result<(), Box<dyn Error>> {
    let crate_search = CrateSearcher::new()?;
    let resp = crate_search.search_sorted_count(term, 1, count as u32, &sort)?;
    print_crates_table(resp)
}

fn print_crates_table(crates: CrateSearchResponse) -> Result<(), Box<dyn Error>> {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.load_preset(TABLE_STYLE);

    table.set_header(vec![
        "Name",
        "Created",
        "Updated",
        "Downloads",
        "Recent Downloads",
        "Max Version",
        "Newest Version",
        "Description",
    ]);

    for crte in crates.crates {
        table.add_row(crte);
    }

    println!("{}", table);

    Ok(())
}

impl ToRow for CrateSearch {
    fn to_row(self) -> Row {
        let desc = self.description.unwrap_or("".to_string());
        Row::from(vec![
            self.name,
            self.created_at.format(STR_FORMAT).to_string(),
            self.updated_at.format(STR_FORMAT).to_string(),
            self.downloads.to_string(),
            self.recent_downloads.to_string(),
            self.max_version,
            self.newest_version,
            desc,
        ])
    }
}
