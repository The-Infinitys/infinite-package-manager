use colored::*;
use std::fmt;
use std::path::PathBuf;

#[derive(thiserror::Error)]
pub enum Error {
    // 組み込みのFromトレイトとthiserrorのマクロを使用
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("I/O Error: {0}")]
    IoError(String), // 新しく追加
    #[error("Deb Sources Parser Error (Deb822): {0}")]
    Deb822(#[from] deb822_lossless::Error),
    #[error("Deb Sources Parse Error (Deb822Parse): {0}")]
    Deb822Parse(#[from] deb822_lossless::ParseError),
    #[error("{0}")]
    Other(String),
    #[error("Base64 decode error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("Parse Error: {0}")]
    ParseError(String),
    #[error("Parse Extension Error: '{0}' is an invalid extension for parsing")]
    ParseExtensionError(String),
    #[error("File Not Found: {0}")]
    FileNotFound(PathBuf),
    #[error("Unsupported operation")]
    Unsupported,
    #[error("Permission denied. This operation requires root privileges.")]
    Permission,
    // 不足していたParseIntErrorを追加
    #[error("Parse Integer Error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Async Runtime Join Error: {0}")]
    AsyncRuntimeJoinError(#[from] tokio::task::JoinError),
    #[error("Serde Yaml Error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("www request error: {0}")]
    WwwReqestError(#[from] reqwest::Error),
    #[error("Signature Verification Error: {0}")]
    SignatureVerificationError(#[from] sequoia_openpgp::Error),
    #[error("Signature Verification Error: {0}")]
    SignatureVerificationAnyHowError(#[from] sequoia_openpgp::anyhow::Error),
    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("No Result (error already reported)")]
    NoResult, // 新しく追加
}

impl Error {
    /// エラーのKindを色付きでFormatterに出力するヘルパー関数
    fn write_kind(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // "kind"というラベルをシアンで出力し、対応するバリアント名を黄色で出力
        let kind_label = "kind".cyan();
        let kind_value = match self {
            Self::Io(_) => "Io",
            Self::IoError(_) => "IoError",
            Self::Deb822(_) => "Deb822",
            Self::Deb822Parse(_) => "Deb822Parse",
            Self::Other(_) => "Other",
            Self::Base64DecodeError(_) => "Base64DecodeError",
            Self::ParseError(_) => "ParseError",
            Self::ParseExtensionError(_) => "ParseExtensionError",
            Self::FileNotFound(_) => "FileNotFound",
            Self::Unsupported => "Unsupported",
            Self::Permission => "Permission",
            Self::ParseIntError(_) => "ParseIntError",
            Self::AsyncRuntimeJoinError(_) => "AsyncRuntimeJoinError",
            Self::SerdeYaml(_) => "SerdeYaml",
            Self::WwwReqestError(_) => "WwwReqestError",
            Self::SignatureVerificationError(_) => "SignatureVerificationError",
            Self::SignatureVerificationAnyHowError(_) => "SignatureVerificationError",
            Self::FromUtf8Error(_) => "FromUtf8Error",
            Self::NoResult => "NoResult",
        };
        // Kindは必ず改行付きで出力します
        writeln!(f, "  {}: {}", kind_label, kind_value.yellow())
    }

    /// エラーの詳細メッセージ（またはパス）を色付きでFormatterに出力するヘルパー関数
    fn write_message(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::FileNotFound(_) => "path".cyan(),
            _ => "message".cyan(),
        };

        // メッセージを格納するための変数。PathBufはdisplay()で文字列として扱う
        let message_value = match self {
            Self::Io(e) => e.to_string(),
            Self::IoError(msg) => msg.clone(),
            Self::Deb822(e) => e.to_string(),
            Self::Deb822Parse(e) => e.to_string(),
            Self::Other(msg) => msg.clone(),
            Self::ParseError(msg) => msg.clone(),
            Self::Base64DecodeError(e) => e.to_string(),
            Self::ParseExtensionError(ext) => {
                format!("'{}' is invalid extension for parse", ext)
            }
            Self::FileNotFound(path) => path.display().to_string(),
            Self::Unsupported => "Unsupported operation".to_string(),
            Self::Permission => {
                "Permission denied. This operation requires root privileges.".to_string()
            }
            Self::ParseIntError(e) => e.to_string(),
            Self::AsyncRuntimeJoinError(e) => e.to_string(),
            Self::SerdeYaml(e) => e.to_string(),
            Self::WwwReqestError(e) => e.to_string(),
            Self::SignatureVerificationError(msg) => msg.to_string(),
            Self::SignatureVerificationAnyHowError(msg) => msg.to_string(),
            Self::FromUtf8Error(e) => e.to_string(),
            Self::NoResult => "No further result available; error handled elsewhere.".to_string(),
        };
        // 最後に、組み立てたメッセージとラベルをFormatterに出力
        // メッセージは赤で強調
        writeln!(f, "  {}: {}", label, message_value.red().italic())
    }
}

// fmt::Debug の実装
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let overwrite_alias = "\x1b\r";
        writeln!(f, "{}{}", overwrite_alias, "Error".red().bold())?;
        self.write_kind(f)?;
        self.write_message(f)
    }
}
