use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::modules::error::UpmError;

pub enum RepositoryType {
    Deb,
    DebSrc,
}
pub struct RepositoryEntry {
    pub url: String,
    pub repo_type: Vec<RepositoryType>,
}
pub fn list(path: &PathBuf) -> Result<RepositoryEntry, UpmError> {
    let file= fs::File::open(path)?;
    let reader = BufReader::new(file);
    let current_block = String::new();
    for line in reader.lines(){
        let line=line?.trim();
    }
    let url = String::from("hello");
    let repo_type = vec![RepositoryType::Deb];
    Ok(RepositoryEntry { url, repo_type })
}
pub fn sources(pathbuf: &PathBuf) {}
