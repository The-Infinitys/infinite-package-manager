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
        };
        // Kindは必ず改行付きで出力します
        writeln!(f, "  {}: {}", kind_label, kind_value.yellow())
    }

    /// エラーの詳細メッセージ（またはパス）を色付きでFormatterに出力するヘルパー関数
    fn write_message(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message_label = "message".cyan();
        let detail_label = "detail".cyan();
        let path_label = "path".cyan();

        // バリアントに応じて、適切なメッセージをラベル付けして出力します
        match self {
            UpmError::Io(e) => {
                write!(f, "  {}: \"{}\"", message_label, e)
            }
            UpmError::Deb822(e) => {
                write!(f, "  {}: \"{}\"", message_label, e)
            }
            UpmError::Deb822Parse(e) => {
                write!(f, "  {}: \"{}\"", message_label, e)
            }
            UpmError::Other(msg) => {
                write!(f, "  {}: \"{}\"", message_label, msg)
            }
            UpmError::ParseError(msg) => {
                write!(f, "  {}: \"{}\"", message_label, msg)
            }
            UpmError::Base64DecodeError(msg) => {
                write!(f, "  {}: \"{}\"", message_label, msg)
            }
            UpmError::ParseExtensionError(ext) => {
                // ラベルを'detail'に変更し、メッセージをカスタム
                write!(
                    f,
                    "  {}: \"{} is invalid extension for parse\"",
                    detail_label, ext
                )
            }
            UpmError::FileNotFound(path) => {
                // ラベルを'path'に変更し、パスを出力
                write!(f, "  {}: \"{}\"", path_label, path.display())
            }
            UpmError::Unsupported => {
                // Unsupportedはkindで全てを表現するため、メッセージを省略します
                Ok(())
            }
            UpmError::Permission => {
                // ラベルを'message'に変更し、メッセージをカスタム
                write!(
                    f,
                    "  {}: \"Permission denied. This operation requires root privileges.\"",
                    message_label,
                )
            }
            UpmError::ParseIntError(e) => {
                write!(f, "  {}: \"{}\"", message_label, e)
            }
        }
    }
}

// fmt::Debug の実装
impl fmt::Debug for UpmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // コンソール出力の特殊なクリアシーケンス
        let overwrite_alias = "\x1b\r";

        // 1. エラータイトルを出力し、自動で改行
        writeln!(f, "{}{}", overwrite_alias, "Error".red().bold())?;

        // 2. write_kindを呼び出し、Kindを出力
        self.write_kind(f)?;

        // 3. write_messageを呼び出し、詳細メッセージを出力
        self.write_message(f)
    }
}
