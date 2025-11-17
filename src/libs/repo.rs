pub mod apt;

pub enum RepositoryEntry {
    Apt(apt::AptRepositoryEntry),
}
