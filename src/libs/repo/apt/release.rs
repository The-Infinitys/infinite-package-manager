use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::{collections::HashMap, path::PathBuf, str::FromStr};

use crate::{
    libs::repo::apt::{AptRepositoryKeyInfo, verify::verification},
    modules::error::Error,
};
fn hex_string_to_vec_u8(hex: &str) -> Result<Vec<u8>, Error> {
    if !hex.len().is_multiple_of(2) {
        return Err(Error::ParseError(
            "Hex string must have an even number of digits.".to_string(),
        ));
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut i = 0;
    while i < hex.len() {
        let byte_str = &hex[i..i + 2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(byte) => bytes.push(byte),
            Err(_) => return Err(Error::ParseError("Invalid hex string".to_string())),
        }
        i += 2;
    }
    Ok(bytes)
}

impl TryFrom<&str> for FileHashMetaData {
    type Error = Error;
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
            Err(Error::ParseError(format!(
                "failed to parse FileHashMetadata, {}",
                value
            )))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub packages_urls: Vec<String>,
    pub fields: HashMap<String, String>,
}

impl AptReleaseInfo {
    pub fn parse_signed(content: &str, key: &AptRepositoryKeyInfo) -> Result<Self, Error> {
        let public_gpg = key.read_owned();
        let verified_context = verification(content, public_gpg)?;
        let verified_context = String::from_utf8(verified_context)?;
        AptReleaseInfo::parse(&verified_context)
    }
    /// 文字列からAptReleaseInfoをパースする
    pub fn parse(content: &str) -> Result<Self, Error> {
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
        let parse_hash_meta = |value: &str| -> Result<Vec<FileHashMetaData>, Error> {
            value
                .split('\n')
                .filter(|line| !line.trim().is_empty())
                .map(FileHashMetaData::try_from)
                .collect::<Result<Vec<FileHashMetaData>, Error>>()
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
                        release_info.packages_urls.extend(
                            release_info
                                .md5sum
                                .iter()
                                .map(|f| f.path.to_string_lossy().to_string()),
                        );
                    }
                    "SHA1" => {
                        release_info.sha1 = parse_hash_meta(value)?;
                        release_info.packages_urls.extend(
                            release_info
                                .sha1
                                .iter()
                                .map(|f| f.path.to_string_lossy().to_string()),
                        );
                    }
                    "SHA256" => {
                        release_info.sha256 = parse_hash_meta(value)?;
                        release_info.packages_urls.extend(
                            release_info
                                .sha256
                                .iter()
                                .map(|f| f.path.to_string_lossy().to_string()),
                        );
                    }
                    _ => {
                        release_info.fields.insert(key, value.to_string());
                    }
                }
            }
        }
        Ok(release_info)
    }
}
#[allow(unused)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Copy)]
pub enum Hash {
    Sha(u32),
    #[default]
    Md5sum,
}

impl Hash {
    /// 与えられたHashTypeとデータ文字列に基づいてハッシュ値を計算します。
    ///
    /// # Arguments
    /// * `hash_type` - 使用するハッシュアルゴリズム
    /// * `data` - ハッシュを計算する対象の文字列
    ///
    /// # Returns
    /// ハッシュ値のバイトベクタ、またはサポートされていないハッシュタイプの場合はエラー
    pub fn _calculate_hash(&self, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        match self {
            Self::Sha(i) => match i {
                256 => {
                    let mut hasher = Sha256::new();
                    hasher.update(data);
                    Ok(hasher.finalize().to_vec())
                }
                512 => {
                    let mut hasher = Sha512::new();
                    hasher.update(data);
                    Ok(hasher.finalize().to_vec())
                }
                _ => Err(Error::Unsupported),
            },
            Self::Md5sum => {
                let digest = md5::compute(data);
                Ok(digest.to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() -> Result<(), Error> {
        let in_release_str = include_str!("../../../../tests/apt/InRelease");
        let release_str = include_str!("../../../../tests/apt/Release");
        let in_release = AptReleaseInfo::parse_signed(in_release_str, &AptRepositoryKeyInfo::None)?;
        let release = AptReleaseInfo::parse(release_str)?;
        assert_eq!(in_release, release);
        Ok(())
    }
}
