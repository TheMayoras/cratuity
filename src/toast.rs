use std::time::{Duration, Instant};

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, StatefulWidget, Widget, Wrap},
};

#[derive(Clone)]
pub struct ToastMessage {}

impl ToastMessage {
    fn get_color(&self, typ: &ToastType) -> Color {
        match typ {
            ToastType::Info => Color::Green,
            ToastType::Warning => Color::Yellow,
            ToastType::Error => Color::Red,
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum ToastType {
    Info,
    Warning,
    Error,
}

impl StatefulWidget for ToastMessage {
    type State = ToastState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let title = state.title.as_ref().map_or("", |t| t.as_str());
        let right = area.right();
        let bottom = area.bottom();
        let len = (25 as u16)
            .max(title.len() as u16)
            .max(state.msg.len() as u16)
            .min(area.width / 4);

        let title = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.get_color(&state.typ)));

        let title_box = Rect {
            x: right - len,
            y: bottom - 5,
            width: len,
            height: 5,
        };
        (Clear {}).render(title_box.clone(), buf);

        let msg_box = title.inner(title_box);
        title.render(title_box, buf);

        Paragraph::new(state.msg.as_str())
            .wrap(Wrap { trim: false })
            .render(msg_box, buf);

        if !state.is_started() {
            state.start();
        }
    }
}

pub struct ToastState {
    /// An optional title to display
    title: Option<String>,
    /// The message to display
    msg: String,
    /// How long to display the message
    dur: Duration,
    /// The time this toast will end
    /// None until the toast is started with `start()`
    end: Option<Instant>,
    typ: ToastType,
}

impl ToastState {
    pub fn new<T: Into<String>>(title: Option<T>, msg: T, dur: Duration, typ: ToastType) -> Self {
        Self {
            title: title.map(|t| t.into()),
            msg: msg.into(),
            dur,
            end: None,
            typ,
        }
    }

    pub fn err<T: Into<String>>(title: Option<T>, msg: T) -> Self {
        Self::new(title, msg, Duration::from_millis(2500), ToastType::Error)
    }

    pub fn start(&mut self) {
        self.end = Some(Instant::now() + self.dur);
    }

    pub fn is_duration_passed(&self) -> bool {
        self.end.map(|end| end < Instant::now()).unwrap_or(false)
    }

    pub fn is_started(&self) -> bool {
        self.end.is_some()
    }
}
