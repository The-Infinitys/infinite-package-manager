use std::fmt::Display;

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
    pub fn load() -> Result<Vec<Self>, UpmError> {
        match system::PackageManager::get() {
            PackageManager::Dpkg => {
                let entries = apt::AptRepositoryEntry::load_all()?;
                Ok(entries.into_iter().map(RepositoryEntry::Apt).collect())
            }
            _ => Err(UpmError::Unsupported),
        }
    }
}

pub fn print_list() -> Result<(), UpmError> {
    let entries = RepositoryEntry::load()?;
    for entry in entries {
        println!("{}", entry);
    }
    Ok(())
}
