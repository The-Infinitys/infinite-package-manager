mod parser;

use deb822_lossless::Deb822;
use std::{collections::HashMap, path::Path, str::FromStr};

use crate::modules::error::UpmError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebPackageEntry {
    pub package: String,
    pub version: String,
    pub architecture: String,
    pub description: String,
    pub installed_size: u64,
    pub maintainer: String,
    pub homepage: Option<String>,
    pub depends: Vec<String>,
    pub status: String,         // e.g., "install ok installed"
    pub priority: String,       // e.g., "optional"
    pub section: String,        // e.g., "devel"
    pub source: Option<String>, // ソースパッケージ名 (バージョン情報を含む場合もある)
    pub replaces: Vec<String>,
    pub provides: Vec<String>,
    pub conflicts: Vec<String>,
    pub pre_depends: Vec<String>,
    pub breaks: Vec<String>,
    pub conffiles: Option<Vec<String>>, // 設定ファイルリスト
    pub original_maintainer: Option<String>,
    pub multi_arch: Option<String>,
    pub recommends: Vec<String>,
    pub suggests: Vec<String>,
    pub enhances: Vec<String>,
    pub essential: Option<String>,
    pub extra_fields: HashMap<String, String>,
}
use colored::*;
use std::fmt;

impl fmt::Display for DebPackageEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_color = if self.status.contains("installed") {
            Color::Green
        } else if self.status.contains("not-installed") {
            Color::Red
        } else {
            Color::BrightBlack
        };

        let key_color = Color::Cyan;

        // 1. ステータスとパッケージ名のヘッダー
        let header = format!("--- Package: {} [{}] ---", self.package.bold(), self.status)
            .color(status_color)
            .bold();
        writeln!(f, "{}", header)?;

        // 2. 基本情報
        writeln!(
            f,
            "{}: {}",
            "Version".color(key_color).bold(),
            self.version.to_string().yellow()
        )?;
        writeln!(
            f,
            "{}: {}",
            "Architecture".color(key_color).bold(),
            self.architecture.to_string().magenta()
        )?;
        writeln!(
            f,
            "{}: {} B",
            "Installed-Size".color(key_color).bold(),
            self.installed_size.to_string().dimmed()
        )?;
        writeln!(
            f,
            "{}: {}",
            "Maintainer".color(key_color).bold(),
            self.maintainer.to_string().blue()
        )?;

        // 3. 説明
        if !self.description.is_empty() {
            writeln!(f, "{}:", "Description".color(key_color).bold(),)?;
            for line in self.description.lines() {
                writeln!(f, "  {}", line.to_string().white())?;
            }
        }

        // 4. ホームページ
        if let Some(homepage) = &self.homepage {
            writeln!(
                f,
                "{}: {}",
                "Homepage".color(key_color).bold(),
                homepage.to_string().underline().white()
            )?;
        }

        // 5. 依存関係
        if !self.depends.is_empty() {
            writeln!(f, "{}:", "Depends".color(key_color).bold(),)?;
            for dep in &self.depends {
                writeln!(f, "  - {}", dep.to_string().dimmed())?;
            }
        }

        // 6. その他のフィールド
        if !self.extra_fields.is_empty() {
            writeln!(f, "{}:", "Other Fields".color(key_color).bold(),)?;
            for (key, value) in &self.extra_fields {
                writeln!(
                    f,
                    "  {}: {}",
                    key.to_string().white(),
                    value.to_string().bright_yellow()
                )?;
            }
        }

        writeln!(f, "{}", "---".color(status_color))
    }
}
impl Default for DebPackageEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl DebPackageEntry {
    pub fn load_from_str(content: &str) -> Result<Self, UpmError> {
        let deb_info = Deb822::from_str(content)?;
        let mut package_entry = DebPackageEntry::new();

        // dpkg -s の出力は単一のパッケージ情報なので、最初のパラグラフのみを処理
        if let Some(info) = deb_info.paragraphs().next() {
            for entry in info.entries() {
                let key = entry.key();
                let value = entry.value();

                if let Some(key) = key {
                    match key.as_str() {
                        "Package" => package_entry.package = value.to_string(),
                        "Version" => package_entry.version = value.to_string(),
                        "Architecture" => package_entry.architecture = value.to_string(),
                        "Description" => package_entry.description = value.to_string(),
                        "Installed-Size" => {
                            package_entry.installed_size = value.parse().unwrap_or(0)
                        }
                        "Maintainer" => package_entry.maintainer = value.to_string(),
                        "Homepage" => package_entry.homepage = Some(value.to_string()),
                        "Depends" => {
                            package_entry.depends =
                                value.split(',').map(|s| s.trim().to_string()).collect();
                        }
                        "Status" => package_entry.status = value.to_string(),
                        "Priority" => package_entry.priority = value.to_string(),
                        "Section" => package_entry.section = value.to_string(),
                        "Source" => package_entry.source = Some(value.to_string()),
                        "Replaces" => {
                            package_entry.replaces =
                                value.split(',').map(|s| s.trim().to_string()).collect()
                        }
                        "Provides" => {
                            package_entry.provides =
                                value.split(',').map(|s| s.trim().to_string()).collect()
                        }
                        "Conflicts" => {
                            package_entry.conflicts =
                                value.split(',').map(|s| s.trim().to_string()).collect()
                        }
                        "Pre-Depends" => {
                            package_entry.pre_depends =
                                value.split(',').map(|s| s.trim().to_string()).collect()
                        }
                        "Breaks" => {
                            package_entry.breaks =
                                value.split(',').map(|s| s.trim().to_string()).collect()
                        }

                        // Conffiles は複数行の場合があるため、値全体を改行で分割してリストにする
                        "Conffiles" => {
                            package_entry.conffiles =
                                Some(value.lines().map(|s| s.trim().to_string()).collect());
                        }
                        _ => {
                            package_entry
                                .extra_fields
                                .insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }

        if package_entry.package.is_empty() {
            Err(UpmError::ParseError(
                "No package information found".to_string(),
            ))
        } else {
            Ok(package_entry)
        }
    }

    pub fn new() -> Self {
        Self {
            package: String::new(),
            version: String::new(),
            architecture: String::new(),
            description: String::new(),
            installed_size: 0,
            maintainer: String::new(),
            homepage: None,
            depends: Vec::new(),
            status: String::new(),
            priority: String::new(),
            section: String::new(),
            source: None,
            replaces: Vec::new(),
            provides: Vec::new(),
            conflicts: Vec::new(),
            pre_depends: Vec::new(),
            breaks: Vec::new(),
            conffiles: None,
            original_maintainer: None,
            multi_arch: None,
            recommends: Vec::new(),
            suggests: Vec::new(),
            enhances: Vec::new(),
            essential: None,
            extra_fields: HashMap::new(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Vec<Self>, UpmError> {
        parser::parse_deb_status_file(path)
    }

    pub fn load_all() -> Result<Vec<Self>, UpmError> {
        let status_file_path = Path::new("/var/lib/dpkg/status");
        Self::load(status_file_path)
    }
}
