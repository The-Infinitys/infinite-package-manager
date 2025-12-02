use colored::*;
use std::fmt;
use std::path::PathBuf;

#[derive(thiserror::Error)]
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
    #[error("Permission denied. This operation requires root privileges.")]
    Permission,
}

impl fmt::Debug for UpmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let overwrite_alias = "\x1b\r";
        // 1. エラータイトルを出力し、自動で改行
        writeln!(f, "{}{}", overwrite_alias, "Error".red().bold())?;

        match self {
            UpmError::Io(e) => {
                // 2. KindとMessageを別々の行に出力
                writeln!(f, "  {}: {}", "kind".cyan(), "Io".yellow())?;
                write!(f, "  {}: \"{}\"", "message".cyan(), e)
            }
            UpmError::Deb822(e) => {
                writeln!(f, "  {}: {}", "kind".cyan(), "Deb822".yellow())?;
                write!(f, "  {}: \"{}\"", "message".cyan(), e)
            }
            UpmError::Deb822Parse(e) => {
                writeln!(f, "  {}: {}", "kind".cyan(), "Deb822Parse".yellow())?;
                write!(f, "  {}: \"{}\"", "message".cyan(), e)
            }
            UpmError::Other(msg) => {
                writeln!(f, "  {}: {}", "kind".cyan(), "Other".yellow())?;
                write!(f, "  {}: \"{}\"", "message".cyan(), msg)
            }
            UpmError::ParseError(msg) => {
                writeln!(f, "  {}: {}", "kind".cyan(), "ParseError".yellow())?;
                write!(f, "  {}: \"{}\"", "message".cyan(), msg)
            }
            UpmError::ParseExtensionError(ext) => {
                writeln!(f, "  {}: {}", "kind".cyan(), "ParseExtensionError".yellow())?;
                write!(
                    f,
                    "  {}: \"{} is invalid extension for parse\"",
                    "detail".cyan(),
                    ext
                )
            }
            UpmError::FileNotFound(path) => {
                writeln!(f, "  {}: {}", "kind".cyan(), "FileNotFound".yellow())?;
                write!(f, "  {}: \"{}\"", "path".cyan(), path.display())
            }
            UpmError::Unsupported => {
                write!(f, "  {}: {}", "kind".cyan(), "Unsupported".yellow())
            }
            UpmError::Permission => {
                writeln!(f, "  {}: {}", "kind".cyan(), "Permission".yellow())?;
                write!(
                    f,
                    "  {}: \"Permission denied. This operation requires root privileges.\"",
                    "message".cyan(),
                )
            }
        }
    }
}
