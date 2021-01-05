use std::{sync::mpsc::Receiver, time::Duration};

use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{
    crates_io::{CrateSearchResponse, CrateSearcher},
    input::InputEvent,
    widgets::{CrateWidget, InputWidget},
};

pub enum AppMode {
    Normal,
    Input(String),
}

pub struct App {
    input_rx: Receiver<InputEvent>,
    client: CrateSearcher,
    pub crates: Option<CrateSearchResponse>,
    pub quit: bool,
    inpt: Option<String>,
    is_inpt: bool,
    page: u32,
    mode: AppMode,
}

impl App {
    pub fn new(input_rx: Receiver<InputEvent>) -> Self {
        Self {
            input_rx,
            client: CrateSearcher::new().unwrap(),
            crates: None,
            quit: false,
            inpt: Some("".to_string()),
            is_inpt: true,
            page: 1,
            mode: AppMode::Input("".to_string()),
        }
    }

    pub fn draw<T: Backend>(&self, f: &mut Frame<T>) {
        let size = f.size();
        let block = Block::default()
            .title("Cratuity (A crates.io quick search TUI)")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);

        let area = block.inner(f.size());

        // render the top message
        let splits = Layout::default()
            .constraints([Constraint::Length(2), Constraint::Min(5)].as_ref())
            .split(area);

        let top = splits[0];
        let area = splits[1];
        let message = if self.is_inpt {
            "Type to enter your search term.  Press Enter to confirm.  Press ESC to cancel"
        } else {
            "Press J/K to move between pages.  Press f to search for a term"
        };
        let message = Paragraph::new(message);
        f.render_widget(message, top);

        // render the bottom message with page details
        let splits = Layout::default()
            .constraints([Constraint::Min(5), Constraint::Length(1)].as_ref())
            .split(area);

        let bot = splits[1];
        let area = splits[0];

        let message = Paragraph::new(format!("Page {}", self.page));
        f.render_widget(message, bot);

        if let Some(CrateSearchResponse { ref crates }) = self.crates {
            let widgets = crates.iter().map(CrateWidget::from);

            let splits = Layout::default()
                .horizontal_margin(1)
                .constraints(
                    [
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                    ]
                    .as_ref(),
                )
                .split(area);
            widgets.zip(splits).for_each(|(w, a)| f.render_widget(w, a));
        }

        f.render_widget(block, size);
        self.draw_mode(f);
    }

    fn draw_mode<T: Backend>(&self, f: &mut Frame<T>) {
        match &self.mode {
            AppMode::Input(msg) => {
                let inpt = InputWidget::new("Enter you search term", msg.as_str());
                f.render_widget(inpt, f.size());
            }
            AppMode::Normal => {}
        }
    }

    pub fn await_input(&mut self) {
        if let Ok(inpt) = self.input_rx.recv_timeout(Duration::from_secs(1)) {
            match &mut self.mode {
                AppMode::Normal => match inpt {
                    InputEvent::Char('q') | InputEvent::Char('Q') => {
                        self.quit = true;
                    }
                    InputEvent::Char('f') | InputEvent::Char('F') => {
                        self.mode = AppMode::Input("".to_string());
                    }
                    InputEvent::Char(c) => match c {
                        'j' | 'J' => {
                            if self.crates.as_ref().map(|c| c.crates.len()).unwrap_or(0) > 0 {
                                self.page += 1;
                                self.do_search();
                            }
                        }
                        'k' | 'K' => {
                            if self.page > 1 {
                                self.page -= 1;
                                self.do_search();
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                AppMode::Input(ref mut msg) => match inpt {
                    InputEvent::Esc => self.mode = AppMode::Normal,
                    InputEvent::Enter => {
                        let replaced = std::mem::take(msg);
                        self.page = 1;
                        self.inpt = Some(replaced);
                        self.do_search();
                        self.mode = AppMode::Normal;
                    }
                    InputEvent::Backspace => {
                        let _ = msg.pop();
                    }
                    InputEvent::Char(c) => msg.push(c),
                },
            }
        }
    }

    fn do_search(&mut self) {
        let search = self.inpt.as_ref();
        let resp = self.client.search(search.unwrap(), self.page);
        match resp {
            Ok(crates) => self.crates = Some(crates),
            Err(_) => self.crates = None,
        }
    }
}
