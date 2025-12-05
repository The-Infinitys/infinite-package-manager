mod packages_parser;
mod parser;
mod release;
mod vec_traits; // Restored
use crate::libs::repo::apt::release::{AptInReleaseInfo, AptReleaseInfo};
use futures::future::join_all;
use reqwest;
use sequoia_openpgp::parse::{PacketParser, PacketParserResult, Parse};
use sequoia_openpgp::{Cert, Packet};
use serde_yaml; // Add this
use sha2::{Digest, Sha256};
// use std::io::Cursor; // 削除
use std::path::Path;
use std::{collections::HashMap, path::PathBuf, process::Command};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
}; // Add this

use crate::{libs::repo::apt::vec_traits::AptRepositoryEntryVec, modules::error::UpmError};
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
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::task;

#[derive(Debug, Clone)]
pub struct InReleaseTarget {
    pub url: String,
    pub local_path: PathBuf,
    pub signed_by_key: AptRepositoryKeyInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagesDownloadTarget {
    pub url: String,
    pub local_path: PathBuf,
    pub hash_type: String, // e.g., "sha256", "md5"
    pub expected_hash: Vec<u8>,
}
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
                        "  {}{}{}",
                        start.red().italic(),
                        "...".dimmed().italic(),
                        end.red().italic()
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

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
impl AptRepositoryKeyInfo {
    // 戻り値を `Option<Vec<u8>>` に変更
    // (ファイルを読み込むため、エラー処理として `Result` を使用することも検討すべきです)
    pub fn read_owned(&self) -> Option<Vec<u8>> {
        match self {
            Self::None => None,
            Self::Bin(bin) => Some(bin.clone()),
            Self::Path(path) => std::fs::read(path).ok(),
        }
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
    pub async fn _load_all_internal(
        apt_sources_dir: impl AsRef<Path>,
        apt_sources_list_dir: impl AsRef<Path>,
    ) -> Result<Vec<Self>, UpmError> {
        let parent_file = apt_sources_dir.as_ref().to_path_buf();
        let parent_dir = apt_sources_list_dir.as_ref().to_path_buf();

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

    pub async fn load_all() -> Result<Vec<Self>, UpmError> {
        let apt_sources_dir = PathBuf::from("/etc/apt/sources.list");
        let apt_sources_list_dir = PathBuf::from("/etc/apt/sources.list.d");
        Self::_load_all_internal(apt_sources_dir, apt_sources_list_dir).await
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

async fn download_file(url: &str, path: &Path) -> Result<(), UpmError> {
    let response = reqwest::get(url).await?;
    let content = response.bytes().await?;

    // 親ディレクトリが存在しない場合は作成
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut file = tokio::fs::File::create(path).await?;
    file.write_all(&content).await?;
    Ok(())
}
async fn verify_signature(
    in_release: &AptInReleaseInfo,
    key_info: &AptRepositoryKeyInfo,
) -> Result<bool, UpmError> {
    let signature_bytes: &[u8] = &in_release.signature;

    // 1. 公開鍵の読み込みとCertの作成
    let pub_key_bytes: Vec<u8> = match key_info.read_owned() {
        Some(key) => key,
        None => return Ok(false), // 鍵がない場合は検証不可
    };
    let cert = Cert::from_bytes(&pub_key_bytes)?;

    // 2. 署名バイナリから Signature パケットをパース
    let mut signature_packet: Option<sequoia_openpgp::packet::Signature> = None;
    let parser = PacketParser::from_bytes(signature_bytes)?;
    // `parser.next()`は`(Option<Packet>, PacketParserResult)`を返す
    // Packetは`enum`なのでパターンマッチで`Signature`を取り出す
    // `PacketParser`のイテレーションと`packet.as_signature()`を使って正しく抽出
    while let PacketParserResult::Some(ref packet) = parser {
        if let Packet::Signature(sig) = &packet.packet {
            signature_packet = Some(sig.clone());
            break;
        }
    }

    let signature = signature_packet.ok_or_else(|| {
        UpmError::Other("Failed to parse Signature packet from InRelease".to_string())
    })?;

    match signature.verify_document(cert.primary_key().key()) {
        Ok(_) => Ok(true), // 検証成功
        Err(e) => {
            // 検証失敗
            eprintln!("PGP Signature verification failed: {}", e);
            Ok(false)
        }
    }
}
async fn _update_internal(
    in_release_cache_dir: PathBuf,
    packages_cache_dir: PathBuf,
    package_list_dir: PathBuf,
) -> Result<(), UpmError> {
    // キャッシュディレクトリとパッケージリストディレクトリが存在することを確認
    tokio::fs::create_dir_all(&in_release_cache_dir).await?;
    tokio::fs::create_dir_all(&packages_cache_dir).await?;
    tokio::fs::create_dir_all(&package_list_dir).await?;

    let entries = AptRepositoryEntry::load_all().await?;
    let in_release_targets = entries.in_release_targets();

    let mut all_packages_targets: Vec<PackagesDownloadTarget> = Vec::new();

    // 1. InReleaseファイルをダウンロードし、検証する
    let in_release_processing_tasks = in_release_targets.into_iter().map(|target| {
        let in_release_cache_dir = in_release_cache_dir.clone();
        let packages_cache_dir = packages_cache_dir.clone();
        async move {
            let local_path = in_release_cache_dir.join(&target.local_path);

            // InReleaseファイルのダウンロード
            download_file(&target.url, &local_path).await?;

            // ダウンロードしたファイルの読み込みとパース
            let content = fs::read_to_string(&local_path).await?;
            let in_release_info = release::AptInReleaseInfo::parse(&content)?;
            // ここで署名の検証を行う
            if !verify_signature(&in_release_info, &target.signed_by_key).await? {
                return Err(UpmError::Unsupported);
            }

            // PackagesDownloadTargetの抽出
            let apt_release_info: AptReleaseInfo = in_release_info.release;
            Ok(apt_release_info.get_packages_download_targets(&target.url, &packages_cache_dir))
        }
    });

    let results: Vec<Result<Vec<PackagesDownloadTarget>, UpmError>> =
        join_all(in_release_processing_tasks).await;

    for result in results {
        match result {
            Ok(targets) => all_packages_targets.extend(targets),
            Err(e) => eprintln!("Error processing InRelease file: {}", e), // エラーは記録するが処理は続行
        }
    }

    // 2. Packagesファイルをダウンロードし、パースし、保存する
    let package_processing_tasks = all_packages_targets.into_iter().map(|target| {
        let package_list_dir = package_list_dir.clone();
        async move {
            let local_path = target.local_path.clone();

            // Packagesファイルのダウンロード
            download_file(&target.url, &local_path).await?;

            // ハッシュ値の検証
            let mut file = File::open(&local_path).await?;
            let mut hasher = Sha256::new();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).await?;
            hasher.update(&buffer);
            let hash_result = hasher.finalize().to_vec();

            if hash_result != target.expected_hash {
                return Err(UpmError::ParseError(format!(
                    "Hash mismatch for Packages file {}. Expected: {:?}, Actual: {:?}",
                    target.url, target.expected_hash, hash_result
                )));
            }

            let packages = packages_parser::parse_packages_file(&local_path)?;
            for pkg in packages {
                let pkg_file_name = format!("{}_{}.yaml", pkg.package, pkg.version);
                let pkg_save_path = package_list_dir.join(pkg_file_name);
                let yaml_content = serde_yaml::to_string(&pkg)?;
                tokio::fs::write(&pkg_save_path, yaml_content.as_bytes()).await?;
            }

            Ok(())
        }
    });

    let _ = join_all(package_processing_tasks).await;

    Ok(())
}

// APTリポジトリのインデックスを非同期に更新する
pub async fn update() -> Result<(), UpmError> {
    let in_release_cache_dir = PathBuf::from("/var/lib/upm/caches/lists/releases");
    let packages_cache_dir = PathBuf::from("/var/lib/upm/caches/lists/packages");
    let package_list_dir = PathBuf::from("/var/lib/upm/repo/packages");
    _update_internal(in_release_cache_dir, packages_cache_dir, package_list_dir).await
}
