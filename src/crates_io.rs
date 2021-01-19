use std::collections::HashMap;

use crate::ceil_div;
use std::str::FromStr;

use chrono::{DateTime, Local};
use reqwest::{blocking::Client, Url};
use serde::{Deserialize, Serialize};

const CRATES_URL: &str = "https://crates.io/api/v1/crates";

#[derive(Clone, Debug, PartialEq, Eq)]
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

impl Default for CratesSort {
    fn default() -> Self {
        Self::Relevance
    }
}

impl FromStr for CratesSort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "relevance" => Ok(Self::Relevance),
            "all time downloaded" | "all-time-downloaded" | "all_time_downloaded" => {
                Ok(Self::Relevance)
            }
            "recent downloaded" | "recent-downloaded" | "recent_downloaded" => {
                Ok(Self::RecentDownload)
            }
            "recent update" | "recent-update" | "recent_update" => Ok(Self::RecentUpdate),
            "newly added" | "newly-added" | "newly_added" => Ok(Self::NewlyAdded),
            _ => Err(format!("Unknown sort method {}", s)),
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
    pub meta: CrateSearchResponseMeta,
    pub crates: Vec<CrateSearch>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateSearchResponseMeta {
    pub total: u32,
    pub next_page: Option<String>,
    pub prev_page: Option<String>,
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
    search_cache: HashMap<(String, String), (u32, HashMap<u32, CrateSearch>)>,
}

impl CrateSearcher {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: Client::builder()
                .user_agent("craters-tui-searcher")
                .build()?,
            search_cache: HashMap::new(),
        })
    }
}
fn get_all_items(
    page_cache: &HashMap<u32, CrateSearch>,
    page: u32,
    items_per_page: u32,
    total_num: u32,
) -> Option<Vec<&CrateSearch>> {
    let start = (page - 1) * items_per_page;
    let end = page * items_per_page;
    let mut res = Vec::with_capacity(items_per_page as usize);
    for index in start..end.min(total_num) {
        res.push(page_cache.get(&index)?);
    }
    Some(res)
}

impl CrateSearcher {
    /// Adds the search query results to the internal cache.
    pub fn search_and_add_to_cache<T: AsRef<str>>(
        &mut self,
        term: T,
        page: u32,
        items_per_page: u32,
        sort: &CratesSort,
    ) -> Result<(), reqwest::Error> {
        let key = (term.as_ref().to_string(), sort.to_sort_string());
        let resp = self.search_sorted(term, page, items_per_page, sort)?;
        let start = (page - 1) * items_per_page;
        let (total, page_cache) = self.search_cache.entry(key).or_insert((0, HashMap::new()));
        for (ind, item) in resp.crates.into_iter().enumerate() {
            page_cache.insert(start + ind as u32, item);
        }
        *total = resp.meta.total;
        Ok(())
    }

    /// Searches the query, defaulting to data available in the cache. If not cached, cache is updated
    /// to include new values. May fetch more items at once to prevent excessive API requests.
    pub fn search_sorted_with_cache<T: AsRef<str>>(
        &mut self,
        term: T,
        page: u32,
        items_per_page: u32,
        sort: &CratesSort,
    ) -> Result<(u32, Vec<&CrateSearch>), reqwest::Error> {
        let key = (term.as_ref().to_string(), sort.to_sort_string());
        if !self.search_cache.contains_key(&key) {
            self.search_and_add_to_cache(
                term.as_ref(),
                ceil_div(page, 10),
                10 * items_per_page,
                sort,
            )?;
            return Ok(self
                .search_sorted_cached(term.as_ref(), page, items_per_page, sort)
                .unwrap());
        }

        if self.check_page_cached(term.as_ref(), page, items_per_page, sort) {
            return Ok(self
                .search_sorted_cached(term.as_ref(), page, items_per_page, sort)
                .unwrap());
        }

        // This thrashes the cache if the page size changes between calls.
        self.search_and_add_to_cache(term.as_ref(), ceil_div(page, 10), 10 * items_per_page, sort)?;
        let (total_num, page_cache) = self.search_cache.get(&key).unwrap();
        let res = get_all_items(page_cache, page, items_per_page, *total_num).unwrap();
        Ok((*total_num, res))
    }

    /// Checks if the given page is cached.
    fn check_page_cached<T: AsRef<str>>(
        &self,
        term: T,
        page: u32,
        items_per_page: u32,
        sort: &CratesSort,
    ) -> bool {
        let key = (term.as_ref().to_string(), sort.to_sort_string());
        if let Some((total_num, page_cache)) = self.search_cache.get(&key) {
            let start = (page - 1) * items_per_page;
            let end = page * items_per_page;
            (start..end.min(*total_num))
                .map(|x| page_cache.get(&x).is_some())
                .all(|x| x)
        } else {
            false
        }
    }

    /// Searches the query and associated pages from the internal cache.
    pub fn search_sorted_cached<T: AsRef<str>>(
        &self,
        term: T,
        page: u32,
        items_per_page: u32,
        sort: &CratesSort,
    ) -> Option<(u32, Vec<&CrateSearch>)> {
        let key = (term.as_ref().to_string(), sort.to_sort_string());
        let (total_num, page_cache) = self.search_cache.get(&key)?;
        Some((
            *total_num,
            get_all_items(page_cache, page, items_per_page, *total_num)?,
        ))
    }

    /// Searches the query without any caching.
    pub fn search_sorted<T: AsRef<str>>(
        &self,
        term: T,
        page: u32,
        items_per_page: u32,
        sort: &CratesSort,
    ) -> Result<CrateSearchResponse, reqwest::Error> {
        self.search_sorted_count(term, page, items_per_page, sort)
    }

    pub fn search_sorted_count<T: AsRef<str>>(
        &self,
        term: T,
        page: u32,
        count: u32,
        sort: &CratesSort,
    ) -> Result<CrateSearchResponse, reqwest::Error> {
        // https://crates.io/api/v1/crates?page=1&per_page=10&q=serde
        let mut url = Url::parse(CRATES_URL).unwrap();
        let url = url
            .query_pairs_mut()
            .append_pair("page", page.to_string().as_str())
            .append_pair("per_page", count.to_string().as_str())
            .append_pair("q", term.as_ref())
            .append_pair("sort", sort.to_sort_string().as_str())
            .finish();

        let req = self.client.get(url.as_str()).build()?;
        let resp = self.client.execute(req)?;
        Ok(resp.json::<CrateSearchResponse>()?)
    }
}
