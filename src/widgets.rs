use std::cmp;

use tui::{
    buffer::Buffer,
    layout::{
        Alignment::{self, Center, Left, Right},
        Constraint,
        Direction::{self, Horizontal},
        Layout, Rect,
    },
    style::{Color, Style},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget, Wrap,
    },
};

use crate::{app::SortingField, crates_io::CrateSearch};

const STR_FORMAT: &str = "%x %H:%M";

pub struct CrateWidget<'a> {
    crte: &'a CrateSearch,
    selected: bool,
}

impl<'a> CrateWidget<'a> {
    pub fn new(crte: &'a CrateSearch, selected: bool) -> Self {
        Self { crte, selected }
    }

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

impl Widget for CrateWidget<'_> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.crte.name.as_str())
            .border_type(BorderType::Plain);

        let block = if self.selected {
            block.border_style(Style::default().fg(Color::Red))
        } else {
            block
        };

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

pub struct InputWidget<'a, T> {
    title: T,
    inpt: &'a str,
}

impl<'a, T: AsRef<str>> InputWidget<'a, T> {
    pub fn new(title: T, inpt: &'a str) -> Self {
        Self { title, inpt }
    }
}
impl<'a, T: AsRef<str>> InputWidget<'a, T> {
    fn get_area(&self, area: Rect) -> Rect {
        let len = self.inpt.len() + 5; // the length needed + some padding + 1 for the '|' character
        let len = len.max(25).max(self.title.as_ref().len()) as u16;

        let horz_pad = (area.width - len).wrapping_div(2);

        let center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(horz_pad),
                    Constraint::Min(len),
                    Constraint::Length(horz_pad),
                ]
                .as_ref(),
            )
            .split(area)[1];

        let height = 5;
        let vert_pad = (area.height - height) / 2;

        let center = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(vert_pad),
                    Constraint::Min(height),
                    Constraint::Length(vert_pad),
                ]
                .as_ref(),
            )
            .split(center)[1];

        center
    }
}

impl<'a, T: AsRef<str>> Widget for InputWidget<'a, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inpt = format!("{}|", self.inpt);
        let inpt = Paragraph::new(inpt);

        let area = self.get_area(area);

        // clear the screen below
        Clear::render(Clear, area, buf);

        // leave a bit of room on the outside
        let area = Block::default().borders(Borders::ALL).inner(area);

        // draw a border around with the given title
        let border = Block::default()
            .title(self.title.as_ref())
            .borders(Borders::ALL);

        let inner = border.inner(area);
        border.render(area, buf);

        inpt.render(inner, buf);
    }
}

pub struct SortingWidget<'a> {
    state: &'a SortingField,
    title: &'a str,
}
impl<'a> SortingWidget<'a> {
    pub fn new(state: &'a SortingField, title: &'a str) -> Self {
        Self { state, title }
    }

    fn get_area(&self, area: Rect) -> Rect {
        let SortingField {
            selection: _selection,
            strs,
            items: _items,
        } = self.state;

        let height = strs.len() as u16 + 2;
        let len = strs.iter().map(String::len).max().unwrap();
        let len = cmp::max(len, self.title.len()) as u16 + 4;

        let horz_pad = (area.width - len as u16).wrapping_div(2);

        let center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(horz_pad),
                    Constraint::Min(len),
                    Constraint::Length(horz_pad),
                ]
                .as_ref(),
            )
            .split(area)[1];

        let vert_pad = (area.height - height) / 2;

        let center = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(vert_pad),
                    Constraint::Min(height),
                    Constraint::Length(vert_pad),
                ]
                .as_ref(),
            )
            .split(center)[1];

        center
    }
}

impl<'a> Widget for SortingWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let SortingField {
            selection,
            strs,
            items: _items,
        } = self.state;
        let mut state = ListState::default();
        state.select(Some(*selection));
        let items = strs.iter().map(String::as_str).map(ListItem::new);
        let list = List::new(items.collect::<Vec<_>>()).highlight_symbol("* ");

        let area = self.get_area(area);
        Clear.render(area, buf);
        let border = Block::default().borders(Borders::ALL).title(self.title);
        let inner = border.inner(area);
        border.render(area, buf);

        StatefulWidget::render(list, inner, buf, &mut state);
    }
}
