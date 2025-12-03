use std::{path::PathBuf, str::FromStr};

use base64::Engine;

use crate::modules::error::UpmError; // UpmErrorのパスは仮定
#[derive(Debug, Clone, Default)]
pub struct AptInReleaseInfo {
    pub signature: Vec<u8>,
    pub release: AptReleaseInfo,
}
impl AptInReleaseInfo {
    pub fn parse(content: &str) -> Result<Self, UpmError> {
        let begin_pgp_signed_message = "-----BEGIN PGP SIGNED MESSAGE-----";
        let begin_pgp_signature = "-----BEGIN PGP SIGNATURE-----";
        let end_pgp_signature = "-----END PGP SIGNATURE-----";
        let content =
            content
                .strip_prefix(begin_pgp_signed_message)
                .ok_or(UpmError::ParseError(format!(
                    "{} not found",
                    begin_pgp_signed_message
                )))?;
        let content = content
            .strip_suffix(end_pgp_signature)
            .ok_or(UpmError::ParseError(format!(
                "{} not found",
                end_pgp_signature
            )))?;
        let s = content
            .split(begin_pgp_signature)
            .into_iter()
            .collect::<Vec<&str>>();
        let (release, signature) = (s[0], s[1]);
        let release = AptReleaseInfo::parse(release)?;
        let mut checksum = "";
        let signature = signature
            .split("\n")
            .map(|s| s.trim())
            .map(|s| {
                if s.starts_with("=") {
                    checksum = s.strip_prefix("=").unwrap();
                    ""
                } else {
                    s
                }
            })
            .collect::<Vec<&str>>()
            .join("");
        println!("{}", signature);
        let b = base64::engine::general_purpose::STANDARD;
        let signature = b.decode(signature)?;
        let checksum = b.decode(checksum)?;
        let crc_24: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_24_OPENPGP);
        let c_checksum = crc_24.checksum(&signature);
        let c_checksum = [
            (c_checksum >> 16) as u8,
            (c_checksum >> 8) as u8,
            c_checksum as u8,
        ]
        .to_vec();
        let checksum_matches = checksum == c_checksum;
        if checksum_matches {
            Ok(Self { release, signature })
        } else {
            let checksum = b.encode(checksum);
            let c_checksum = b.encode(c_checksum);
            Err(UpmError::ParseError(format!(
                "Signature doesn't match. expected: {}, actual: {}",
                checksum, c_checksum
            )))
        }
    }
}
fn hex_string_to_vec_u8(hex: &str) -> Result<Vec<u8>, UpmError> {
    if hex.len() % 2 != 0 {
        return Err(UpmError::ParseError(
            "Hex string must have an even number of digits.".to_string(),
        ));
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut i = 0;
    while i < hex.len() {
        let byte_str = &hex[i..i + 2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(byte) => bytes.push(byte),
            Err(_) => return Err(UpmError::ParseError("Invalid hex string".to_string())),
        }
        i += 2;
    }
    Ok(bytes)
}

impl TryFrom<&str> for FileHashMetaData {
    type Error = UpmError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        let contents = value.split_ascii_whitespace().collect::<Vec<&str>>();
        if contents.len() == 3 {
            let hash = contents[0];
            let size = contents[1];
            let path = contents[2];
            let hash = hex_string_to_vec_u8(hash)?;
            let size = u64::from_str(size)?;
            let path = PathBuf::from(path);
            Ok(Self { hash, size, path })
        } else {
            Err(UpmError::ParseError(format!(
                "failed to parse FileHashMetadata, {}",
                value
            )))
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileHashMetaData {
    pub hash: Vec<u8>,
    pub size: u64,
    pub path: PathBuf,
}
impl Default for FileHashMetaData {
    fn default() -> Self {
        Self {
            hash: Vec::new(),
            size: 0,
            path: PathBuf::new(),
        }
    }
}
#[derive(Debug, Default, Clone)]
pub struct AptReleaseInfo {
    pub origin: String,
    pub label: String,
    pub suite: String,
    pub version: String,
    pub codename: String,
    pub date: String,
    pub architectures: Vec<String>,
    pub components: Vec<String>,
    pub description: String,
    pub md5sum: Vec<FileHashMetaData>,
    pub sha1: Vec<FileHashMetaData>,
    pub sha256: Vec<FileHashMetaData>,
}

impl AptReleaseInfo {
    /// 文字列からAptReleaseInfoをパースする
    pub fn parse(content: &str) -> Result<Self, UpmError> {
        let mut release_info = AptReleaseInfo::default();
        let deb822_data = deb822_lossless::Deb822::from_str(content)?;

        // ヘルパー関数: スペース区切りの文字列をVec<String>に変換
        let parse_space_separated_list = |value: &str| -> Vec<String> {
            value
                .split_ascii_whitespace()
                .map(|s| s.to_string())
                .collect()
        };

        // ヘルパー関数: Hashメタデータをパース
        let parse_hash_meta = |value: &str| -> Result<Vec<FileHashMetaData>, UpmError> {
            value
                .split('\n')
                .filter(|line| !line.trim().is_empty())
                .map(FileHashMetaData::try_from)
                .collect::<Result<Vec<FileHashMetaData>, UpmError>>()
        };

        // Deb822形式は基本的に単一のパラグラフで構成されるため、最初のパラグラフのみを処理
        let paragraphs = deb822_data.paragraphs();
        for paragraph in paragraphs {
            for key in paragraph.keys() {
                let value = paragraph.get(key.as_str()).unwrap();
                let value = value.trim();
                match key.as_str() {
                    // フィールド名が'Original' -> 'Origin'の可能性を考慮し、テストに合わせる
                    "Origin" => {
                        release_info.origin = value.to_string();
                    }
                    "Label" => {
                        release_info.label = value.to_string();
                    }
                    "Suite" => {
                        release_info.suite = value.to_string();
                    }
                    "Version" => {
                        release_info.version = value.to_string();
                    }
                    "Codename" => {
                        release_info.codename = value.to_string();
                    }
                    "Date" => {
                        release_info.date = value.to_string();
                    }
                    "Architectures" => {
                        release_info.architectures = parse_space_separated_list(value);
                    }
                    "Components" => {
                        release_info.components = parse_space_separated_list(value);
                    }
                    "Description" => {
                        release_info.description = value.to_string();
                    }
                    "MD5Sum" => {
                        release_info.md5sum = parse_hash_meta(value)?;
                    }
                    "SHA1" => {
                        release_info.sha1 = parse_hash_meta(value)?;
                    }
                    "SHA256" => {
                        release_info.sha256 = parse_hash_meta(value)?;
                    }
                    _ => {}
                }
            }
        }
        Ok(release_info)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() -> Result<(), UpmError> {
        let target = include_str!("../../../../tests/apt/InRelease");
        let in_release = AptInReleaseInfo::parse(target)?;
        println!("{:#?}", in_release);
        Ok(())
    }
}
