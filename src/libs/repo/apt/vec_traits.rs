use crate::{libs::repo::apt::AptRepositoryEntry, modules::error::UpmError};

pub trait AptRepositoryVecExt {
    fn precheck(&self) -> Result<Vec<AptRepositoryEntry>, UpmError>;
}

impl AptRepositoryVecExt for Vec<AptRepositoryEntry> {
    fn precheck(&self) -> Result<Vec<AptRepositoryEntry>, UpmError> {
        let changed_repos = Vec::new();
        Ok(changed_repos)
    }
}
