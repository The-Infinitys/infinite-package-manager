use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{
    libs::system::{self, PackageManager},
    modules::error::UpmError,
};

mod apt;

pub enum RepositoryEntry {
    Apt(apt::AptRepositoryEntry),
}
impl Display for RepositoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apt(apt_repository_entry) => write!(f, "{}", apt_repository_entry),
        }
    }
}
impl RepositoryEntry {
    pub(crate) async fn _load_internal(
        apt_sources_dir: impl AsRef<Path>,
        apt_sources_list_dir: impl AsRef<Path>,
    ) -> Result<Vec<Self>, UpmError> {
        match system::PackageManager::get() {
            PackageManager::Dpkg => {
                let entries = apt::AptRepositoryEntry::_load_all_internal(
                    apt_sources_dir,
                    apt_sources_list_dir,
                )
                .await?;
                Ok(entries.into_iter().map(RepositoryEntry::Apt).collect())
            }
            _ => Err(UpmError::Unsupported),
        }
    }
    pub async fn load() -> Result<Vec<Self>, UpmError> {
        let apt_sources_dir = PathBuf::from("/etc/apt/sources.list");
        let apt_sources_list_dir = PathBuf::from("/etc/apt/sources.list.d");
        Self::_load_internal(apt_sources_dir, apt_sources_list_dir).await
    }
}

pub async fn _print_list_internal(
    apt_sources_dir: impl AsRef<Path>,
    apt_sources_list_dir: impl AsRef<Path>,
) -> Result<(), UpmError> {
    let entries = RepositoryEntry::_load_internal(apt_sources_dir, apt_sources_list_dir).await?;
    for entry in entries {
        println!("{}", entry);
    }
    Ok(())
}

pub async fn print_list() -> Result<(), UpmError> {
    let apt_sources_dir = PathBuf::from("/etc/apt/sources.list");
    let apt_sources_list_dir = PathBuf::from("/etc/apt/sources.list.d");
    _print_list_internal(apt_sources_dir, apt_sources_list_dir).await
}

pub async fn update() -> Result<(), UpmError> {
    let output = Command::new("id").arg("-u").output()?;
    let uid = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u32>()
        .map_err(|e| UpmError::Other(format!("Failed to parse UID: {}", e)))?;

    if uid != 0 {
        return Err(UpmError::Permission);
    }

    match system::PackageManager::get() {
        PackageManager::Dpkg => {
            apt::update().await
        }
        _ => Err(UpmError::Unsupported),
    }
}
