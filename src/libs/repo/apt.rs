pub mod parser;

// Key/Value形式のオプションを保持する型を定義します
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum AptRepositoryType {
    #[default]
    Deb,
    DebSrc,
}
impl TryFrom<&str> for AptRepositoryType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "deb" => Ok(Self::Deb),
            "deb-src" => Ok(Self::DebSrc),
            _ => Err(String::new()),
        }
    }
}

#[derive(Debug, Clone)]

pub enum AptRepositoryKeyInfo {
    Path(PathBuf),
    Bin(Vec<u8>),
}
#[derive(Debug, Clone)]
pub struct AptRepositoryEntry {
    pub repo_type: Vec<AptRepositoryType>,
    pub uris: String,
    pub suites: Vec<String>,
    pub components: Vec<String>,
    pub enabled: bool, // エントリがコメントアウトされていないか
    pub signed_by: AptRepositoryKeyInfo,
    pub options: HashMap<String, String>,
}
impl AptRepositoryEntry {
    pub fn new() -> Self {
        let repo_type = vec![];
        let uris = String::new();
        let suites = vec![];
        let components = vec![];
        let enabled = false;
        let signed_by = AptRepositoryKeyInfo::Bin(vec![]);
        let options = HashMap::new();
        Self {
            repo_type,
            uris,
            suites,
            components,
            enabled,
            signed_by,
            options,
        }
    }
}
