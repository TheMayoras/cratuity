use std::collections::VecDeque;
use std::error::Error;
use std::{cmp, sync::mpsc::Receiver, time::Duration};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

#[cfg(feature = "copy")]
use clipboard::{ClipboardContext, ClipboardProvider};

use crate::toast::ToastState;
use crate::{crates_io::CrateSearch, toast::ToastMessage};

use crate::{
    ceil_div,
    crates_io::{CrateSearcher, CratesSort},
    input::InputEvent,
    widgets::{CrateWidget, InputWidget, SortingWidget},
};

pub struct SortingField {
    pub(crate) selection: usize,
    pub(crate) items: Vec<CratesSort>,
    pub(crate) strs: Vec<String>,
}

impl From<&'_ CratesSort> for SortingField {
    fn from(sort: &'_ CratesSort) -> Self {
        let mut items = Vec::with_capacity(4);
        items.push(CratesSort::Relevance);
        items.push(CratesSort::AllTimeDownload);
        items.push(CratesSort::RecentDownload);
        items.push(CratesSort::RecentUpdate);
        items.push(CratesSort::NewlyAdded);

        let selection = items.iter().position(|item| sort.eq(item)).unwrap();
        let strs = items.iter().map(|item| format!("{}", item)).collect();

        Self {
            selection,
            items,
            strs,
        }
    }
}

pub enum AppMode {
    Normal,
    Input(String),
    Sorting(SortingField),
}

pub struct App {
    input_rx: Receiver<InputEvent>,
    client: CrateSearcher,
    pub quit: bool,
    inpt: Option<String>,
    page: u32,
    items_per_page: u32,
    sort: CratesSort,
    mode: AppMode,
    selection: Option<usize>,
    /// a queue of toast messages to display to the user
    toast: VecDeque<ToastState>,
}

impl App {
    pub fn new(input_rx: Receiver<InputEvent>) -> Self {
        Self {
            input_rx,
            client: CrateSearcher::new().unwrap(),
            quit: false,
            inpt: Some("".to_string()),
            page: 1,
            items_per_page: 5,
            mode: AppMode::Input("".to_string()),
            sort: CratesSort::Relevance,
            selection: None,
            toast: VecDeque::new(),
        }
    }

    pub fn draw<T: Backend>(&mut self, f: &mut Frame<T>) {
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
        let message = match self.mode {
            AppMode::Normal => {
                Text::raw("Press N/P to move between pages.  Press f to search for a term\nPress J/K to change the highlighted Crate and press C to copy it's Cargo.toml string") 
            }
            AppMode::Input(_) => {
                "Type to enter your search term.  Press Enter to confirm.  Press ESC to cancel".into()
            }
            AppMode::Sorting(_) => {
                "Press J/K to move between options.  Press Enter to confirm.  Press ESC to cancel".into()
            }
        };
        let message = Paragraph::new(message);
        f.render_widget(message, top);

        // render the bottom message with page details
        let splits = Layout::default()
            .constraints([Constraint::Min(5), Constraint::Length(1)].as_ref())
            .split(area);

        let bot = splits[1];
        let area = splits[0];

        if let Some((total, crates)) = self.get_cached_crates() {
            let message =
                Paragraph::new(format!("Page {} of {}", self.page, self.num_pages(total)));
            f.render_widget(message, bot);

            let mut widgets = Vec::new();
            for (i, crte) in crates.iter().enumerate() {
                if let Some(selection) = self.selection {
                    widgets.push(CrateWidget::new(crte, selection == i));
                } else {
                    widgets.push(CrateWidget::new(crte, false));
                }
            }

            let mut raw = vec![100u16 / (self.items_per_page as u16); self.items_per_page as usize];
            let sum_diff = 100 - raw.iter().sum::<u16>();
            if sum_diff != 0 {
                for item in raw[..sum_diff as usize].iter_mut() {
                    *item += 1;
                }
            }
            let splits = Layout::default()
                .horizontal_margin(1)
                .constraints(
                    raw.into_iter()
                        .map(Constraint::Percentage)
                        .collect::<Vec<_>>(),
                )
                .split(area);
            widgets
                .into_iter()
                .zip(splits)
                .for_each(|(w, a)| f.render_widget(w, a));
        }

        f.render_widget(block, size);
        self.draw_mode(f);

        self.toast
            .front_mut()
            .map(|toast| f.render_stateful_widget(ToastMessage {}, size, toast));
    }

    fn draw_mode<T: Backend>(&self, f: &mut Frame<T>) {
        match &self.mode {
            AppMode::Input(msg) => {
                let inpt = InputWidget::new("Enter your search term", msg.as_str());
                f.render_widget(inpt, f.size());
            }
            AppMode::Normal => {}
            AppMode::Sorting(state) => {
                let widget = SortingWidget::new(state, "Select your sorting method");
                f.render_widget(widget, f.size());
            }
        }
    }

    fn next_item(&mut self) {
        if let Some(selection) = self.selection {
            if let Some((_, crates)) = self.get_cached_crates() {
                if selection + 1 >= crates.len() {
                    self.next_page();
                } else {
                    self.selection = Some(selection + 1);
                }
            }
        }
    }

    fn prev_item(&mut self) {
        if let Some(selection) = self.selection {
            self.selection = if selection == 0 && self.page != 1 {
                self.prev_page();
                if let Some((_, crates)) = self.get_cached_crates() {
                    Some(crates.len().saturating_sub(1))
                } else {
                    None
                }
            } else {
                Some(selection.saturating_sub(1))
            }
        }
    }

    fn next_page(&mut self) {
        if let Some((total, _)) = self.get_cached_crates() {
            if self.page * self.items_per_page < total {
                self.selection = Some(0);
                self.page += 1;
                self.do_search();
            }
        }
    }

    fn prev_page(&mut self) {
        if self.page > 1 {
            self.selection = Some(0);
            self.page -= 1;
            self.do_search();
        }
    }

    fn home(&mut self) {
        self.selection = Some(0);
        if self.page != 1 {
            self.page = 1;
            self.do_search();
        }
    }

    fn num_pages(&self, num_items: u32) -> u32 {
        ceil_div(num_items, self.items_per_page)
    }

    fn end(&mut self) {
        if let Some((total, _)) = self.get_cached_crates() {
            if self.page * self.items_per_page < total {
                self.page = self.num_pages(total);
                self.do_search();
            }
        }
        if let Some((_, crates)) = self.get_cached_crates() {
            self.selection = crates.len().checked_sub(1);
        }
    }

    pub fn await_input(&mut self) {
        if let Ok(inpt) = self.input_rx.recv_timeout(Duration::from_secs(1)) {
            match &mut self.mode {
                AppMode::Normal => match inpt {
                    InputEvent::Char(c) => match c {
                        'f' | 'F' => self.mode = AppMode::Input("".to_string()),
                        'q' | 'Q' => self.quit = true,
                        'n' | 'N' => self.next_page(),
                        'p' | 'P' => self.prev_page(),
                        'j' | 'J' => self.next_item(),
                        'k' | 'K' => self.prev_item(),
                        's' | 'S' => self.mode = AppMode::Sorting(SortingField::from(&self.sort)),
                        'o' | 'O' => {
                            if let Err(msg) = self.open_selection() {
                                self.toast.push_back(ToastState::err(
                                    Some("Cannot open in browser"),
                                    format!("{}", msg).as_str(),
                                ))
                            }
                        }
                        'c' | 'C' => {
                            if let Err(msg) = self.copy_selection() {
                                self.toast.push_back(ToastState::err(
                                    Some("Clipboard Error".to_string()),
                                    format!("{}", msg),
                                ))
                            };
                        }
                        _ => {}
                    },
                    InputEvent::Down => {
                        self.next_item();
                    }
                    InputEvent::Up => {
                        self.prev_item();
                    }
                    InputEvent::Right | InputEvent::PageDown => {
                        self.next_page();
                    }
                    InputEvent::Left | InputEvent::PageUp => {
                        self.prev_page();
                    }
                    InputEvent::End => {
                        self.end();
                    }
                    InputEvent::Home => {
                        self.home();
                    }

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
                    _ => {}
                },
                AppMode::Sorting(SortingField {
                    selection,
                    items,
                    strs: _,
                }) => match inpt {
                    InputEvent::Esc => self.mode = AppMode::Normal,
                    InputEvent::Enter => {
                        self.sort = items[*selection].clone();
                        self.page = 1;
                        self.mode = AppMode::Normal;
                        self.do_search();
                    }
                    InputEvent::Right
                    | InputEvent::Down
                    | InputEvent::Char('j')
                    | InputEvent::Char('J') => {
                        *selection = cmp::min(*selection + 1, 4);
                    }
                    InputEvent::Left
                    | InputEvent::Up
                    | InputEvent::Char('k')
                    | InputEvent::Char('K') => {
                        *selection = selection.saturating_sub(1);
                    }
                    InputEvent::Char(c) => match c {
                        'p' | 'P' => {
                            *selection = selection.saturating_sub(1);
                        }
                        'n' | 'N' => {
                            *selection = cmp::min(*selection + 1, 4);
                        }
                        _ => {}
                    },
                    _ => {}
                },
            }
        }

        if let Some(toast) = self.toast.front() {
            if toast.is_started() && toast.is_duration_passed() {
                self.toast.pop_front();
            }
        }
    }

    fn get_cached_crates(&self) -> Option<(u32, Vec<&CrateSearch>)> {
        let search = self.inpt.as_ref();
        self.client.search_sorted_cached(
            search.unwrap(),
            self.page,
            self.items_per_page,
            &self.sort,
        )
    }

    fn do_search(&mut self) {
        let search = self.inpt.as_ref();
        let (_, crates) = self
            .client
            .search_sorted_with_cache(search.unwrap(), self.page, self.items_per_page, &self.sort)
            .unwrap_or((0, vec![]));
        self.selection = if crates.is_empty() { None } else { Some(0) }
    }

    #[cfg(feature = "browser")]
    fn open_selection(&self) -> Result<(), Box<dyn Error>> {
        if let Some(selection) = self.selection {
            if let Some((_, crates)) = self.get_cached_crates() {
                if let Some(crte) = crates.get(selection) {
                    if let Some(docs) = &crte.documentation {
                        open::that(docs).map_err(|err| {
                            Box::<dyn Error>::from(format!(
                                "Error opening documentation in browser.\n{}",
                                err
                            ))
                        })?
                    } else {
                        Err(Box::<dyn Error>::from(
                            "No documentation link for is given for this crate!",
                        ))?
                    }
                }
            }
        }

        Ok(())
    }
    #[cfg(not(feature = "browser"))]
    fn open_selection(&self) -> Result<(), Box<dyn Error>> {
        Err(Box::<dyn Error>::from("Feature Disabled"))
    }

    #[cfg(feature = "copy")]
    fn copy_selection(&self) -> Result<(), Box<dyn Error>> {
        if let Some(selection) = self.selection {
            if let Some((_, crates)) = self.get_cached_crates() {
                let toml = crates
                    .get(selection)
                    .map(|x| CrateSearch::get_toml_str(x.to_owned()));
                let mut clipboard: ClipboardContext = ClipboardProvider::new()
                    .map_err(|_err| Box::<dyn Error>::from("Error setting clipboard contents"))?;

                if let Some(toml) = toml {
                    clipboard.set_contents(toml).map_err(|_err| {
                        Box::<dyn Error>::from("Error setting clipboard contents")
                    })?;
                }
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "copy"))]
    fn copy_selection(&self) -> Result<(), Box<dyn Error>> {
        Err("Feature Disabled".into())
    }
}
