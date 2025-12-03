mod parser;
mod release;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use crate::modules::error::UpmError;
use base64::Engine;
use parser::list;
use parser::sources;
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum AptRepositoryType {
    #[default]
    Deb,
    DebSrc,
}
use colored::*;
use std::fmt;
use tokio::task;
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
        for repo_type in &self.repo_types {
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
            AptRepositoryKeyInfo::Bin(bin) => {
                let b = &base64::engine::general_purpose::STANDARD;
                let encoded_key = b.encode(bin);

                if encoded_key.len() > 16 {
                    // 文字列が16文字より長い場合 (先頭8文字 + 末尾8文字 + 省略記号)
                    let start = &encoded_key[..8];
                    let end = &encoded_key[encoded_key.len() - 8..];

                    // 省略形式で表示
                    writeln!(
                        f,
                        "  {}",
                        format!("{}{}{}", start.red().italic(), "...".dimmed().italic(),end.red().italic()).to_string() // 色はPathに合わせて赤に
                    )?;
                } else {
                    // 文字列が短い場合は全体を表示
                    writeln!(f, "  {}", encoded_key.to_string().red().italic())?;
                }
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
    Bin(Vec<u8>),
    None,
}
#[derive(Debug, Clone)]
pub struct AptRepositoryEntry {
    pub repo_types: Vec<AptRepositoryType>,
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
        // NOTE: dpkgコマンドの実行はブロッキングI/Oであり、非同期コンテキスト外で実行することが推奨されるため、
        // ここでは便宜上そのままにしています。理想的には、この情報もメインスレッドの初期化で取得すべきです。
        let architectures = match Command::new("dpkg").arg("--print-architecture").output() {
            Ok(output) if output.status.success() => {
                let arch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if arch.is_empty() { vec![] } else { vec![arch] }
            }
            _ => {
                eprintln!(
                    "Warning: Failed to determine native architecture using 'dpkg --print-architecture'. Falling back to empty architecture list."
                );
                vec![]
            }
        };
        let signed_by = AptRepositoryKeyInfo::None;
        let options = HashMap::new();
        Self {
            repo_types: repo_type,
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
    pub async fn load_all() -> Result<Vec<Self>, UpmError> {
        let parent_file = PathBuf::from("/etc/apt/sources.list");
        let parent_dir = PathBuf::from("/etc/apt/sources.list.d");

        // 複数の非同期タスクの結果を格納するためのベクタ
        let mut tasks = Vec::new();

        // 1. /etc/apt/sources.list の読み込みタスクを生成
        if parent_file.exists() {
            let file_path = parent_file.clone();
            tasks.push(task::spawn(async move {
                task::spawn_blocking(move || Self::load(&file_path)).await
            }));
        }

        // 2. /etc/apt/sources.list.d/ ディレクトリ内のファイルの読み込みタスクを生成
        if parent_dir.is_dir() {
            match tokio::fs::read_dir(parent_dir).await {
                Ok(mut dir) => {
                    while let Some(entry) = dir.next_entry().await? {
                        let path = entry.path();

                        // ファイルであるか、拡張子が適切かのチェック
                        if path.is_file() {
                            let ext_is_valid = path
                                .extension()
                                .map(|ext| {
                                    let s = ext.to_string_lossy();
                                    s == "list" || s == "sources"
                                })
                                .unwrap_or(false);

                            if ext_is_valid {
                                // 各ファイルのパース処理を独立した非同期タスクとして登録
                                tasks.push(task::spawn(async move {
                                    task::spawn_blocking(move || AptRepositoryEntry::load(&path))
                                        .await
                                }));
                            }
                        }
                    }
                }
                // ディレクトリが存在しないか読み込みエラーの場合は、エラーを返すかスキップ
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => { /* スキップ */ }
                Err(e) => return Err(e.into()), // その他のI/Oエラーは返す
            }
        }

        let results = futures::future::join_all(tasks).await;
        let all_entries: Vec<Vec<Self>> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .filter_map(Result::ok)
            .flatten()
            .collect();
        let all_entries = all_entries.into_iter().flatten().collect();
        Ok(all_entries)
    }
    /// 個々のリポジトリ設定から、ダウンロード対象となるベースURLを生成する
    fn parent_urls(&self) -> Vec<String> {
        let architectures = &self.architectures;
        self.repo_types
            .iter()
            .flat_map(|repo_type| {
                self.suites.iter().flat_map(move |suite| {
                    self.components.iter().flat_map(move |component| {
                        match repo_type {
                            AptRepositoryType::Deb => {
                                // Debタイプの場合、アーキテクチャの数だけURLを生成
                                architectures
                                    .iter()
                                    .map(move |architecture| {
                                        format!(
                                            "{}/dists/{}/{}/binary-{}",
                                            self.uris, suite, component, architecture
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            }
                            AptRepositoryType::DebSrc => {
                                // DebSrcタイプの場合、アーキテクチャに依存せず1つのURLを生成
                                vec![format!(
                                    "{}/dists/{}/{}/source",
                                    self.uris, suite, component
                                )]
                            }
                        }
                    })
                })
            })
            .collect::<Vec<String>>()
    }

    /// 各ベースURLから、Packages.gz などの実際のダウンロードURLを生成する
    pub fn target_urls(&self, filename: &str) -> Vec<String> {
        self.parent_urls()
            .iter()
            .map(|parent| format!("{}/Packages{}", parent, filename))
            .collect()
    }
}

/// APTリポジトリのインデックスを非同期に更新する
pub async fn update() -> Result<(), UpmError> {
    Ok(())
}
