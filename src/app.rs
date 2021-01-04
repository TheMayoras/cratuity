use std::{sync::mpsc::Receiver, time::Duration};

use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    widgets::{Block, BorderType, Borders},
    Frame,
};

use crate::{
    crates_io::{CrateSearchResponse, CrateSearcher},
    input::InputEvent,
    widgets::{CrateWidget, InputWidget},
};

pub struct App {
    input_rx: Receiver<InputEvent>,
    client: CrateSearcher,
    pub crates: Option<CrateSearchResponse>,
    pub quit: bool,
    inpt: Option<String>,
    is_inpt: bool,
    page: u32,
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
        }
    }

    pub fn draw<T: Backend>(&self, f: &mut Frame<T>) {
        let size = f.size();
        let block = Block::default()
            .title("Craters (A crates.io quick search TUI)")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);

        if let Some(CrateSearchResponse { ref crates }) = self.crates {
            let widgets = crates.iter().map(CrateWidget::from);

            let splits = Layout::default()
                .horizontal_margin(1)
                .constraints(
                    [
                        Constraint::Percentage(15),
                        Constraint::Percentage(15),
                        Constraint::Percentage(15),
                        Constraint::Percentage(15),
                        Constraint::Percentage(15),
                    ]
                    .as_ref(),
                )
                .split(block.inner(f.size()));
            widgets.zip(splits).for_each(|(w, a)| f.render_widget(w, a));
        }

        f.render_widget(block, size);

        // render an input widget
        if self.is_inpt {
            if let Some(inpt) = &self.inpt {
                let inpt = InputWidget::new("Enter you search term", inpt.as_str());
                f.render_widget(inpt, f.size());
            }
        }
    }

    pub fn await_input(&mut self) {
        match self.input_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(InputEvent::Char('q')) | Ok(InputEvent::Char('Q')) if !self.is_inpt => {
                self.quit = true
            }
            Ok(InputEvent::Char('f')) | Ok(InputEvent::Char('F')) if !self.is_inpt => {
                self.inpt = Some("".to_string());
                self.is_inpt = true;
            }
            Ok(InputEvent::Esc) if self.is_inpt => self.is_inpt = false,
            Ok(InputEvent::Backspace) if self.is_inpt => {
                let _ = self.inpt.as_mut().map(|i| i.pop());
            }
            Ok(InputEvent::Enter) if self.is_inpt => {
                self.is_inpt = false;
                let search = self.inpt.as_ref();
                let resp = self.client.search(search.unwrap(), self.page);
                match resp {
                    Ok(crates) => self.crates = Some(crates),
                    Err(_) => self.crates = None,
                }
            }
            Ok(InputEvent::Char(c)) if self.is_inpt => {
                self.inpt.as_mut().map(|i| i.push(c));
            }
            Ok(InputEvent::Char(c)) if !self.is_inpt => match c {
                'j' | 'J' => {
                    if self.crates.as_ref().map(|c| c.crates.len()).unwrap_or(0) > 0 {
                        let search = self.inpt.as_ref();
                        self.page += 1;
                        let resp = self.client.search(search.unwrap(), self.page);
                        match resp {
                            Ok(crates) => self.crates = Some(crates),
                            Err(_) => self.crates = None,
                        }
                    }
                }
                'k' | 'K' => {
                    if self.crates.as_ref().map(|c| c.crates.len()).unwrap_or(0) > 0
                        && self.page > 1
                    {
                        let search = self.inpt.as_ref();
                        self.page -= 1;
                        let resp = self.client.search(search.unwrap(), self.page);
                        match resp {
                            Ok(crates) => self.crates = Some(crates),
                            Err(_) => self.crates = None,
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
