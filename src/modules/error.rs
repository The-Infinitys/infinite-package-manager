use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum UpmError {
    #[error("Io Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Deb Sources Parser Error: {0}")]
    Deb822(#[from] deb822_lossless::Error),
    #[error("Deb Sources Parser Error: {0}")]
    Deb822Parse(#[from] deb822_lossless::ParseError),
    #[error("{0}")]
    Other(String),
    #[error("Parse Error: {0}")]
    ParseError(String),
    #[error("Parse Error: {0} is invalid extension for parse")]
    ParseExtensionError(String),
    #[error("File Not Found: {0}")]
    FileNotFound(PathBuf),
    #[error("Unsupported execution")]
    Unsupported,
}
