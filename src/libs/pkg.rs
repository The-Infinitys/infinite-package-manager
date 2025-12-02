use std::fmt::Display;

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
    pub fn load() -> Result<Vec<Self>, UpmError> {
        match system::PackageManager::get() {
            PackageManager::Dpkg => {
                let entries = deb::DebPackageEntry::load_all()?;
                Ok(entries
                    .into_iter()
                    .map(PackageEntry::Deb)
                    .collect())
            }
            _ => Err(UpmError::Unsupported),
        }
    }
}

pub fn print_list() -> Result<(), UpmError> {
    let entries = PackageEntry::load()?;
    for entry in entries {
        println!("{}", entry);
    }
    Ok(())
}
