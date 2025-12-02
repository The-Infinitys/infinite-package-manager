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
use colored::*;
use std::fmt;
// coloredクレートのColorizeトレイトをスコープに持ち込む

// AptRepositoryTypeにDisplayを実装（coloredを使用しない部分）
impl fmt::Display for AptRepositoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AptRepositoryType::Deb => write!(f, "deb"),
            AptRepositoryType::DebSrc => write!(f, "deb-src"),
        }
    }
}

impl fmt::Display for AptRepositoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_color = if self.enabled {
            Color::Green
        } else {
            Color::BrightBlack
        };

        let status_text = if self.enabled { "ENABLED" } else { "DISABLED" };

        let key_color = Color::Cyan;

        // 1. ステータス行
        let header = format!("--- Repository [{}] ---", status_text)
            .color(status_color)
            .bold();
        writeln!(f, "{}", header)?;

        // 2. 基本情報
        writeln!(
            f,
            "{}: {}",
            "Enabled".color(key_color).bold(),
            self.enabled.to_string().color(status_color)
        )?;

        writeln!(
            f,
            "{}: {}",
            "URI".color(key_color).bold(),
            self.uris.to_string().yellow()
        )?;

        // 3. リスト形式のフィールド

        // Types:
        writeln!(f, "{}:", "Types".color(key_color).bold(),)?;
        for repo_type in &self.repo_type {
            writeln!(f, "  - {}", repo_type.to_string().dimmed())?;
        }

        // Suites:
        writeln!(f, "{}:", "Suites".color(key_color).bold(),)?;
        for suite in &self.suites {
            writeln!(f, "  - {}", suite.to_string().green())?;
        }

        // Components:
        writeln!(f, "{}:", "Components".color(key_color).bold(),)?;
        for component in &self.components {
            writeln!(f, "  - {}", component.to_string().blue())?;
        }

        // Architectures: (新しく追加)
        if !self.architectures.is_empty() {
            writeln!(f, "{}:", "Architectures".color(key_color).bold())?;
            for arch in &self.architectures {
                writeln!(f, "  - {}", arch.to_string().magenta())?;
            }
        } else {
            writeln!(
                f,
                "{}: {}",
                "Architectures".color(key_color).bold(),
                "all".to_string().magenta().dimmed()
            )?;
        }

        // 4. キー情報 (SignedBy)
        writeln!(f, "{}:", "SignedBy".color(key_color).bold(),)?;
        match &self.signed_by {
            AptRepositoryKeyInfo::Path(path) => {
                writeln!(f, "  {}", path.display().to_string().red().italic())?;
            }
            AptRepositoryKeyInfo::None => {
                let display_text = "[No Key Specified]".to_string();
                writeln!(f, "  {}", display_text.red().dimmed().italic())?;
            }
        }

        // 5. オプション
        if !self.options.is_empty() {
            writeln!(f, "{}:", "Options".color(key_color).bold(),)?;
            for (key, value) in &self.options {
                writeln!(
                    f,
                    "  {}: {}",
                    key.to_string().white(),
                    value.to_string().bright_yellow()
                )?;
            }
        } else {
            writeln!(
                f,
                "{}: {}",
                "Options".color(key_color).bold(),
                "{}".dimmed()
            )?;
        }
        writeln!(f, "{}", "---".color(status_color))
    }
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
    None,
}
#[derive(Debug, Clone)]
pub struct AptRepositoryEntry {
    pub repo_type: Vec<AptRepositoryType>,
    pub uris: String,
    pub suites: Vec<String>,
    pub components: Vec<String>,
    pub enabled: bool,
    pub signed_by: AptRepositoryKeyInfo,
    pub architectures: Vec<String>,
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
        let architectures = vec![];
        let signed_by = AptRepositoryKeyInfo::None;
        let options = HashMap::new();
        Self {
            repo_type,
            uris,
            suites,
            components,
            enabled,
            signed_by,
            architectures,
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
