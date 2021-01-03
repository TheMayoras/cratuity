use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CrateSearchLinks {
    pub version_downloads: String,
    pub versions: String,
    pub owners: String,
    pub owner_team: String,
    pub owner_user: String,
    pub reverse_dependencies: String,
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
