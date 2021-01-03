use std::{sync::mpsc::Sender, time::Duration};

use crossterm::event::{self, Event as TermEvent, KeyCode};

pub enum InputEvent {
    Quit,
}

pub struct InputMonitor {
    tx: Sender<InputEvent>,
}

impl InputMonitor {
    pub fn new(tx: Sender<InputEvent>) -> Self {
        InputMonitor { tx }
    }

    pub fn monitor(&self) {
        loop {
            if let Ok(true) = event::poll(Duration::from_secs(10)) {
                if let TermEvent::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            self.tx.send(InputEvent::Quit).unwrap()
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
