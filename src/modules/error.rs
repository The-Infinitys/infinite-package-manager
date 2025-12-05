use colored::*;
use std::fmt;
use std::path::PathBuf;

#[derive(thiserror::Error)]
pub enum UpmError {
    // 組み込みのFromトレイトとthiserrorのマクロを使用
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
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
    SignatureVerificationError(#[from] sequoia_openpgp::anyhow::Error),
}

impl UpmError {
    /// エラーのKindを色付きでFormatterに出力するヘルパー関数
    fn write_kind(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // "kind"というラベルをシアンで出力し、対応するバリアント名を黄色で出力
        let kind_label = "kind".cyan();
        let kind_value = match self {
            UpmError::Io(_) => "Io",
            UpmError::Deb822(_) => "Deb822",
            UpmError::Deb822Parse(_) => "Deb822Parse",
            UpmError::Other(_) => "Other",
            UpmError::Base64DecodeError(_) => "Base64DecodeError",
            UpmError::ParseError(_) => "ParseError",
            UpmError::ParseExtensionError(_) => "ParseExtensionError",
            UpmError::FileNotFound(_) => "FileNotFound",
            UpmError::Unsupported => "Unsupported",
            UpmError::Permission => "Permission",
            UpmError::ParseIntError(_) => "ParseIntError",
            UpmError::AsyncRuntimeJoinError(_) => "AsyncRuntimeJoinError",
            UpmError::SerdeYaml(_) => "SerdeYaml",
            UpmError::WwwReqestError(_) => "WwwReqestError",
            UpmError::SignatureVerificationError(_) => "SignatureVerificationError",
        };
        // Kindは必ず改行付きで出力します
        writeln!(f, "  {}: {}", kind_label, kind_value.yellow())
    }

    /// エラーの詳細メッセージ（またはパス）を色付きでFormatterに出力するヘルパー関数
    fn write_message(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            UpmError::FileNotFound(_) => "path".cyan(),
            _ => "message".cyan(),
        };

        // メッセージを格納するための変数。PathBufはdisplay()で文字列として扱う
        let message_value = match self {
            UpmError::Io(e) => e.to_string(),
            UpmError::Deb822(e) => e.to_string(),
            UpmError::Deb822Parse(e) => e.to_string(),
            UpmError::Other(msg) => msg.clone(),
            UpmError::ParseError(msg) => msg.clone(),
            UpmError::Base64DecodeError(e) => e.to_string(),
            UpmError::ParseExtensionError(ext) => {
                format!("'{}' is invalid extension for parse", ext)
            }
            UpmError::FileNotFound(path) => path.display().to_string(),
            UpmError::Unsupported => "Unsupported operation".to_string(),
            UpmError::Permission => {
                "Permission denied. This operation requires root privileges.".to_string()
            }
            UpmError::ParseIntError(e) => e.to_string(),
            UpmError::AsyncRuntimeJoinError(e) => e.to_string(),
            UpmError::SerdeYaml(e) => e.to_string(),
            UpmError::WwwReqestError(e) => e.to_string(),
            UpmError::SignatureVerificationError(msg) => msg.to_string(),
        };
        // 最後に、組み立てたメッセージとラベルをFormatterに出力
        // メッセージは赤で強調
        writeln!(f, "  {}: {}", label, message_value.red().italic())
    }
}

// fmt::Debug の実装
impl fmt::Debug for UpmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let overwrite_alias = "\x1b\r";
        writeln!(f, "{}{}", overwrite_alias, "Error".red().bold())?;
        self.write_kind(f)?;
        self.write_message(f)
    }
}
