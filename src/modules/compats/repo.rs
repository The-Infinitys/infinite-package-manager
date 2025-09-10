/// src/modules/compats/repo.rs
/// レポジトリ関係の互換レイヤー
pub mod apt;

/// Represents the type of a package repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoType {
    /// Advanced Package Tool (APT) repository, commonly used in Debian-based systems.
    Apt,
    // Add other repository types here as needed (e.g., Yum, Zypper, etc.)
}

/// Represents a generic package repository with common attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    /// A unique name or identifier for the repository.
    pub name: String,
    /// The base URL of the repository.
    pub url: String,
    /// A list of components or sections within the repository (e.g., "main", "universe").
    pub components: Vec<String>,
    /// A list of architectures supported by the repository (e.g., "amd64", "i386").
    pub architectures: Vec<String>,
    /// Optional: The GPG key ID or fingerprint used to sign packages from this repository.
    pub key: Option<String>,
    /// The type of the repository.
    pub repo_type: RepoType,
}

impl Repository {
    /// Creates a new `Repository` instance.
    pub fn new(
        name: String,
        url: String,
        components: Vec<String>,
        architectures: Vec<String>,
        key: Option<String>,
        repo_type: RepoType,
    ) -> Self {
        Repository {
            name,
            url,
            components,
            architectures,
            key,
            repo_type,
        }
    }
}
