use tui::{
    buffer::Buffer,
    layout::{
        Alignment::{self, Center, Left, Right},
        Constraint,
        Direction::{self, Horizontal},
        Layout, Rect,
    },
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, Wrap},
};

use crate::crates_io::CrateSearch;

const STR_FORMAT: &str = "%x %H:%M";

pub struct CrateWidget<'a> {
    crte: &'a CrateSearch,
}

impl<'a> CrateWidget<'a> {
    fn render_top(&self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().fg(Color::Red);
        let parts = Layout::default()
            .direction(Horizontal)
            .constraints(
                [
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ]
                .as_ref(),
            )
            .split(area);
        let paragraph = format!(
            "Created: {}",
            self.crte.created_at.format(STR_FORMAT).to_string()
        );
        let paragraph = Paragraph::new(paragraph.as_str())
            .style(style.clone())
            .alignment(Left);
        paragraph.render(parts[0], buf);

        let paragraph = format!(
            "Updated: {}",
            self.crte.updated_at.format(STR_FORMAT).to_string()
        );
        let paragraph = Paragraph::new(paragraph.as_str())
            .style(style.clone())
            .alignment(Center);
        paragraph.render(parts[1], buf);

        let paragraph = format!("Downloads: {}", self.crte.downloads);
        let paragraph = Paragraph::new(paragraph.as_str())
            .style(style.clone())
            .alignment(Center);
        paragraph.render(parts[2], buf);

        let paragraph = format!("Recent Downloads: {}", self.crte.recent_downloads);
        let paragraph = Paragraph::new(paragraph.as_str())
            .style(style.clone())
            .alignment(Right);
        paragraph.render(parts[3], buf);
    }

    fn render_versions(&self, area: Rect, buf: &mut Buffer) {
        let style = Style::default().fg(Color::Blue);

        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ]
                .as_ref(),
            )
            .split(area);

        let max_ver = format!("Max Version: {}", self.crte.max_version);
        Paragraph::new(max_ver.as_str())
            .style(style)
            .render(sections[0], buf);

        let recent_ver = format!("Newest Version: {}", self.crte.newest_version);
        Paragraph::new(recent_ver.as_str())
            .style(style)
            .alignment(Alignment::Center)
            .render(sections[1], buf);
    }
}

impl<'a> From<&'a CrateSearch> for CrateWidget<'a> {
    fn from(crte: &'a CrateSearch) -> Self {
        CrateWidget { crte }
    }
}

impl Widget for CrateWidget<'_> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.crte.name.as_str())
            .border_type(BorderType::Plain);

        let inner = block.inner(area); // the inner area that the border does not take up
        block.render(area, buf);

        let sections = Layout::default()
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Percentage(100),
                ]
                .as_ref(),
            )
            .split(inner);

        // Created at date
        self.render_top(sections[0], buf);
        self.render_versions(sections[1], buf);

        // Crate description
        if let Some(ref desc) = self.crte.description {
            let paragraph = Paragraph::new(desc.as_str()).wrap(Wrap { trim: true });
            paragraph.render(sections[2], buf);
        }
    }
}
