mod parser;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    libs::repo::apt::parser::{list, sources},
    modules::error::UpmError,
};

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
impl Default for AptRepositoryEntry {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn load(path: impl AsRef<Path>) -> Result<Vec<Self>, UpmError> {
        let path = path.as_ref();
        let ext = path.extension();
        match ext {
            Some(ext) => {
                let ext = ext.to_string_lossy();
                match ext.as_ref() {
                    "sources" => sources(path),
                    "list" => list(path),
                    _ => Err(UpmError::ParseExtensionError(format!("\".{}\"", ext))),
                }
            }
            None => Err(UpmError::ParseExtensionError("None".to_string())),
        }
    }
    pub fn load_all() -> Result<Vec<Self>, UpmError> {
        let parent_file = Path::new("/etc/apt/sources.list");
        let parent_dir = Path::new("/etc/apt/sources.list.d");

        let mut all_entries: Vec<Self> = Vec::new();

        // 1. /etc/apt/sources.list の読み込み
        // ファイルが存在し、読み込みに成功した場合のみ処理
        if parent_file.exists() {
            match AptRepositoryEntry::load(parent_file) {
                Ok(entries) => all_entries.extend(entries),
                // sources.listのパースエラーは致命的ではない場合があるが、ここではエラーを返す
                Err(e) => return Err(e),
            }
        }

        // 2. /etc/apt/sources.list.d/ ディレクトリ内のファイルの読み込み
        if parent_dir.exists() && parent_dir.is_dir() {
            // ディレクトリ内のエントリを走査
            for entry in std::fs::read_dir(parent_dir)? {
                let entry = entry?;
                let path = entry.path();

                // ファイルであり、適切な拡張子を持つかチェック
                // .list または .sources で終わるファイルのみを対象とするのが一般的です
                if path.is_file() {
                    let ext_is_valid = path
                        .extension()
                        .map(|ext| {
                            let s = ext.to_string_lossy();
                            s == "list" || s == "sources"
                        })
                        .unwrap_or(false);

                    if ext_is_valid {
                        // 個々のファイルを読み込み、成功したエントリを追加
                        match AptRepositoryEntry::load(&path) {
                            Ok(entries) => all_entries.extend(entries),
                            // 個別のファイルのエラーはスキップせずにエラーを返す設計にする
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
        }

        Ok(all_entries)
    }
}
