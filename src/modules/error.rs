#[derive(thiserror::Error, Debug)]
pub enum UpmError {
    #[error("Io Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Deb Sources Parser Error: {0}")]
    Deb822(#[from] deb822_lossless::Error),
    #[error("{0}")]
    Other(String),
    #[error("Parse Error: {0}")]
    ParseError(String),
    #[error("Parse Error: {0} is invalid extension for parse")]
    ParseExtensionError(String),
    #[error("Unsupported execution")]
    Unsupported,
}
