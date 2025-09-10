/// src/modules/compats/repo.rs
/// レポジトリ関係の互換レイヤー
mod apt;

/// Represents the type of a package repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoInfo {
    /// Advanced Package Tool (APT) repository, commonly used in Debian-based systems.
    Apt(apt::RepoInfo),
    // Add other repository types here as needed (e.g., Yum, Zypper, etc.)
}

/// Represents a generic package repository with common attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    /// A unique name or identifier for the repository.
    pub name: String,
    /// The base URL of the repository.
    pub url: String,
    /// The type of the repository.
    pub info: RepoInfo,
}

impl Repository {
    pub fn new(name: &str, url: &str, info: RepoInfo) -> Self {
        let name = name.to_string();
        let url = url.to_string();
        Self { name, url, info }
    }
}
