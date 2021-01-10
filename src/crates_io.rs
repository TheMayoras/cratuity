use std::thread;

use chrono::{DateTime, Local};
use crossbeam_channel::{self, bounded, Receiver, Sender};
use reqwest::{blocking::Client, Url};
use serde::{Deserialize, Serialize};

use crate::input::InputEvent;

const CRATES_URL: &str = "https://crates.io/api/v1/crates";

#[derive(Clone, PartialEq, Eq)]
pub enum CratesSort {
    Relevance,
    AllTimeDownload,
    RecentDownload,
    RecentUpdate,
    NewlyAdded,
}

impl CratesSort {
    pub fn to_sort_string(&self) -> String {
        match self {
            CratesSort::Relevance => "relevance".to_string(),
            CratesSort::AllTimeDownload => "downloads".to_string(),
            CratesSort::RecentDownload => "recent-downloads".to_string(),
            CratesSort::RecentUpdate => "recent-updates".to_string(),
            CratesSort::NewlyAdded => "new".to_string(),
        }
    }
}

impl std::fmt::Display for CratesSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CratesSort::Relevance => f.write_str("Relevance"),
            CratesSort::AllTimeDownload => f.write_str("All Time Downloads"),
            CratesSort::RecentDownload => f.write_str("Recent Downloads"),
            CratesSort::RecentUpdate => f.write_str("Recently Updated"),
            CratesSort::NewlyAdded => f.write_str("Newly Added"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateSearchResponse {
    pub crates: Vec<CrateSearch>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateSearch {
    pub id: String,
    pub name: String,
    pub updated_at: DateTime<Local>,
    pub created_at: DateTime<Local>,
    pub downloads: u64,
    pub recent_downloads: u64,
    pub max_version: String,
    pub newest_version: String,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub repository: Option<String>,
    pub links: CrateSearchLinks,
    pub exact_match: bool,
}

impl CrateSearch {
    #[cfg(not(feature = "no-copy"))]
    pub fn get_toml_str(&self) -> String {
        format!("{} = \"{}\"", self.id, self.newest_version)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateSearchLinks {
    pub version_downloads: String,
    pub versions: String,
    pub owners: String,
    pub owner_team: String,
    pub owner_user: String,
    pub reverse_dependencies: String,
}

struct CrateSearcherWorker {
    tx: Sender<InputEvent>,
    rx: Receiver<(String, u32, CratesSort)>,
    client: Client,
}

impl CrateSearcherWorker {
    fn new(tx: Sender<InputEvent>, rx: Receiver<(String, u32, CratesSort)>) -> Self {
        Self {
            tx,
            rx,
            client: Client::builder()
                .user_agent("craters-tui-searcher")
                .build()
                .unwrap(),
        }
    }

    fn run(&self) -> Result<(), reqwest::Error> {
        loop {
            if let Ok((term, page, sort)) = self.rx.recv() {
                let mut url = Url::parse(CRATES_URL).unwrap();
                let url = url
                    .query_pairs_mut()
                    .append_pair("page", page.to_string().as_str())
                    .append_pair("per_page", "5")
                    .append_pair("q", term.as_str())
                    .append_pair("sort", sort.to_sort_string().as_str())
                    .finish();

                let req = self.client.get(url.as_str()).build()?;
                let res = self.client.execute(req).map(|resp| {
                    resp.json::<CrateSearchResponse>()
                        .expect("Unable to deserialize the crate search response")
                });
                if let Ok(res) = res {
                    self.tx.send(InputEvent::Results(res)).ok();
                }
            }
        }
    }
}

/// A struct that will be used to search crates.io
pub struct CrateSearcher {
    tx: Sender<(String, u32, CratesSort)>,
}

impl CrateSearcher {
    pub fn new(input_tx: Sender<InputEvent>) -> Result<Self, reqwest::Error> {
        let (tx, rx) = bounded(1);
        thread::spawn(move || CrateSearcherWorker::new(input_tx, rx).run());
        Ok(Self { tx })
    }
}

impl CrateSearcher {
    pub fn search_sorted<T: Into<String>>(&self, term: T, page: u32, sort: &CratesSort) {
        // https://crates.io/api/v1/crates?page=1&per_page=10&q=serde
        self.tx.send((term.into(), page, sort.clone())).unwrap();
    }
}
