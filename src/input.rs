use std::{
    error::Error,
    io,
    io::{Stdout, Write},
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};


use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as TermEvent, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};




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
