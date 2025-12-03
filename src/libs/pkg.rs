use std::fmt::Display;
use std::path::Path;

use crate::{
    libs::system::{self, PackageManager},
    modules::error::UpmError,
};

pub mod deb;

pub enum PackageEntry {
    Deb(deb::DebPackageEntry),
}
impl Display for PackageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deb(deb_package_entry) => write!(f, "{}", deb_package_entry),
        }
    }
}
impl PackageEntry {
    pub(crate) fn _load_internal(
        status_file_path: impl AsRef<Path>,
    ) -> Result<Vec<Self>, UpmError> {
        match system::PackageManager::get() {
            PackageManager::Dpkg => {
                let entries = deb::DebPackageEntry::_load_all_internal(status_file_path)?;
                Ok(entries.into_iter().map(PackageEntry::Deb).collect())
            }
            _ => Err(UpmError::Unsupported),
        }
    }

    pub fn load() -> Result<Vec<Self>, UpmError> {
        let status_file_path = Path::new("/var/lib/dpkg/status");
        Self::_load_internal(status_file_path)
    }
}

pub(crate) fn _print_list_internal(status_file_path: impl AsRef<Path>) -> Result<(), UpmError> {
    let entries = PackageEntry::_load_internal(status_file_path)?;
    for entry in entries {
        println!("{}", entry);
    }
    Ok(())
}

pub fn print_list() -> Result<(), UpmError> {
    let status_file_path = Path::new("/var/lib/dpkg/status");
    _print_list_internal(status_file_path)
}
