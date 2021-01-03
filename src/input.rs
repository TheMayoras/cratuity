use std::{sync::mpsc::Sender, time::Duration};

use crossterm::event::{self, Event as TermEvent, KeyCode};

pub enum InputEvent {
    Char(char),
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
                        KeyCode::Char(c @ 'q') | KeyCode::Char(c @ 'Q') => {
                            self.tx.send(InputEvent::Char(c)).unwrap()
                        }
                        KeyCode::Char(c) => self.tx.send(InputEvent::Char(c)).unwrap(),
                        _ => {}
                    }
                }
            }
        }
    }
}
