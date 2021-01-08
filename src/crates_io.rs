use chrono::{DateTime, Local};
use reqwest::{blocking::Client, Url};
use serde::{Deserialize, Serialize};

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

/// A struct that will be used to search crates.io
pub struct CrateSearcher {
    client: Client,
}

impl CrateSearcher {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: Client::builder()
                .user_agent("craters-tui-searcher")
                .build()?,
        })
    }
}

impl CrateSearcher {
    pub fn search_sorted<T: AsRef<str>>(
        &self,
        term: T,
        page: u32,
        sort: &CratesSort,
    ) -> Result<CrateSearchResponse, reqwest::Error> {
        // https://crates.io/api/v1/crates?page=1&per_page=10&q=serde
        let mut url = Url::parse(CRATES_URL).unwrap();
        let url = url
            .query_pairs_mut()
            .append_pair("page", page.to_string().as_str())
            .append_pair("per_page", "5")
            .append_pair("q", term.as_ref())
            .append_pair("sort", sort.to_sort_string().as_str())
            .finish();

        let req = self.client.get(url.as_str()).build()?;
        self.client.execute(req).map(|resp| {
            resp.json::<CrateSearchResponse>()
                .expect("Unable to deserialize the crate search response")
        })
    }
}
