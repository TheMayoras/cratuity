use std::{sync::mpsc::Sender, time::Duration};

use crossterm::event::{self, Event, KeyCode};

pub enum InputEvent {
    Char(char),
    Esc,
    Enter,
    Backspace,
    Right,
    Left,
    Resize,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
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
                match event::read().unwrap() {
                    Event::Key(key) => match key.code {
                        KeyCode::Esc => self.tx.send(InputEvent::Esc).unwrap(),
                        KeyCode::Enter => self.tx.send(InputEvent::Enter).unwrap(),
                        KeyCode::Backspace => self.tx.send(InputEvent::Backspace).unwrap(),
                        KeyCode::Right => self.tx.send(InputEvent::Right).unwrap(),
                        KeyCode::Left => self.tx.send(InputEvent::Left).unwrap(),
                        KeyCode::Up => self.tx.send(InputEvent::Up).unwrap(),
                        KeyCode::Down => self.tx.send(InputEvent::Down).unwrap(),
                        KeyCode::PageUp => self.tx.send(InputEvent::PageUp).unwrap(),
                        KeyCode::PageDown => self.tx.send(InputEvent::PageDown).unwrap(),
                        KeyCode::Home | KeyCode::Char('g') => {
                            self.tx.send(InputEvent::Home).unwrap()
                        }
                        KeyCode::End | KeyCode::Char('G') => self.tx.send(InputEvent::End).unwrap(),
                        KeyCode::Char(c) => self.tx.send(InputEvent::Char(c)).unwrap(),
                        _ => {}
                    },
                    Event::Mouse(_) => {}
                    Event::Resize(_, _) => {
                        self.tx.send(InputEvent::Resize).unwrap();
                    }
                }
            }
        }
    }
}
