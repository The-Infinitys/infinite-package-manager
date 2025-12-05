use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    libs::repo::apt::{AptRepositoryKeyInfo, verify::verification},
    modules::error::UpmError,
};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AptInReleaseInfo;
impl AptInReleaseInfo {
    pub fn parse(content: &str, key: &AptRepositoryKeyInfo) -> Result<AptReleaseInfo, UpmError> {
        let public_gpg = key.read_owned();
        let verified_context = verification(content, public_gpg)?;
        let verified_context = String::from_utf8(verified_context)?;
        AptReleaseInfo::parse(&verified_context)
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
    pub fields: HashMap<String, String>,
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
                    _ => {
                        release_info.fields.insert(key, value.to_string());
                    }
                }
            }
        }
        Ok(release_info)
    }

    pub fn get_packages_download_targets(
        &self,
        base_url: &str,
        packages_cache_dir: &Path,
    ) -> Vec<crate::libs::repo::apt::PackagesDownloadTarget> {
        let mut targets = Vec::new();

        let mut add_targets = |hash_list: &[FileHashMetaData], hash_type: &str| {
            for meta in hash_list {
                let file_name = meta.path.to_string_lossy();
                // Check if the file is a Packages file (could be Packages, Packages.gz, Packages.xz, etc.)
                // For now, only consider Packages and Packages.gz
                if file_name.ends_with("Packages") || file_name.ends_with("Packages.gz") {
                    let url = format!(
                        "{}/{}",
                        base_url.rsplit_once('/').unwrap_or((base_url, "")).0,
                        file_name
                    );
                    let local_path = packages_cache_dir.join(&meta.path);
                    targets.push(crate::libs::repo::apt::PackagesDownloadTarget {
                        url,
                        local_path,
                        hash_type: hash_type.to_string(),
                        expected_hash: meta.hash.clone(),
                    });
                }
            }
        };

        // Prefer SHA256, then SHA1, then MD5
        add_targets(&self.sha256, "sha256");
        add_targets(&self.sha1, "sha1");
        add_targets(&self.md5sum, "md5");

        targets
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
    pub fn _calculate_hash(&self, data: impl AsRef<[u8]>) -> Result<Vec<u8>, UpmError> {
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
                _ => Err(UpmError::Unsupported),
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
    fn parse_test() -> Result<(), UpmError> {
        let in_release_str = include_str!("../../../../tests/apt/InRelease");
        let release_str = include_str!("../../../../tests/apt/Release");
        let in_release = AptInReleaseInfo::parse(in_release_str, &AptRepositoryKeyInfo::None)?;
        let release = AptReleaseInfo::parse(release_str)?;
        assert_eq!(in_release, release);
        Ok(())
    }
}
