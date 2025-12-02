mod parser;

use std::{
    collections::HashMap,
    path::Path,
    str::FromStr,
};
use deb822_lossless::Deb822;

use crate::modules::error::UpmError;

#[derive(Debug, Clone)]
pub struct DebPackageEntry {
    pub package: String,
    pub version: String,
    pub architecture: String,
    pub description: String,
    pub installed_size: u64,
    pub maintainer: String,
    pub homepage: Option<String>,
    pub depends: Vec<String>,
    pub status: String, // e.g., "install ok installed"
    // その他のフィールドを必要に応じて追加
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
                            package_entry.depends = value
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .collect();
                        }
                        "Status" => package_entry.status = value.to_string(),
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
            Err(UpmError::ParseError("No package information found".to_string()))
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
