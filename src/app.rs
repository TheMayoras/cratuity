use std::{sync::mpsc::Receiver, time::Duration};

use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    widgets::{Block, BorderType, Borders},
    Frame,
};

use crate::{crates_io::CrateSearchResponse, input::InputEvent, widgets::CrateWidget};

pub struct App {
    input_rx: Receiver<InputEvent>,
    pub crates: Option<CrateSearchResponse>,
    pub quit: bool,
}

impl App {
    pub fn new(input_rx: Receiver<InputEvent>) -> Self {
        Self {
            input_rx,
            crates: None,
            quit: false,
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
        }

        f.render_widget(block, size);
    }

    pub fn await_input(&mut self) {
        match self.input_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(InputEvent::Char('q')) | Ok(InputEvent::Char('Q')) => self.quit = true,
            _ => {}
        }
    }
}
