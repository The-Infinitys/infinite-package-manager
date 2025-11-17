#[derive(thiserror::Error, Debug)]
pub enum UpmError {
    #[error("Io Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Deb Sources Parser Error: {0}")]
    Deb822(#[from] deb822_lossless::Error),
}
