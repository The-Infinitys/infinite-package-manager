use super::AptRepositoryEntry;
use std::collections::HashSet;
use std::path::PathBuf;

/// AptRepositoryEntryのベクタに対する拡張トレイト
pub trait AptRepositoryEntryVec {
    /// 各リポジトリエントリから、InReleaseファイルをダウンロードするためのターゲット情報リストを生成する
    ///
    /// 戻り値: Vec<InReleaseTarget>
    fn in_release_targets(&self) -> Vec<super::InReleaseTarget>;
}

// AptRepositoryEntryのベクタ（Vec<AptRepositoryEntry>）にトレイトを実装
impl AptRepositoryEntryVec for Vec<AptRepositoryEntry> {
    fn in_release_targets(&self) -> Vec<super::InReleaseTarget> {
        let mut unique_targets = HashSet::new();
        self.iter()
            // 1. 有効なエントリのみを対象とする
            .filter(|entry| entry.enabled)
            // 2. 各エントリを、複数のSuiteに対応するInRelease URLのストリームに展開する
            .for_each(|entry| {
                let signed_by_key = entry.signed_by.clone(); // Clone the key info
                entry.suites.iter().for_each(|suite| {
                    // InReleaseファイルの標準的なパス形式: URI/dists/SUITE/InRelease
                    let url = format!("{}/dists/{}/InRelease", entry.uris, suite);

                    let mut packages_urls: Vec<String> = Vec::new();
                    packages_urls.extend(entry.target_urls(".xz"));

                    unique_targets.insert((url, signed_by_key.clone(), packages_urls)); // Store URL, key info and packages URLs
                });
            });

        // 3. 一意のURLに対して、ローカル保存ファイル名を生成する
        unique_targets
            .into_iter()
            .map(|(url, signed_by_key, packages_urls)| {
                // ローカルファイル名を作成 (ユーザー要望の形式: archive.ubuntu.com_ubuntu_dists_suite_InRelease)
                let mut local_name = url.clone();

                // 1. スキーム (http://, https://) を削除
                if let Some(stripped) = local_name.strip_prefix("http://") {
                    local_name = stripped.to_string();
                } else if let Some(stripped) = local_name.strip_prefix("https://") {
                    local_name = stripped.to_string();
                }

                // 2. 最後の "/InRelease" を削除
                // InReleaseファイル自体が末尾に来ることを想定
                if let Some(stripped) = local_name.strip_suffix("/InRelease") {
                    local_name = stripped.to_string();
                }

                // 3. 残ったパスセパレータ "/" を "_" に置換
                let filename_body = local_name.replace('/', "_");

                // 4. 拡張子 "_InRelease" を再付加
                let local_filename = format!("{}_InRelease", filename_body);

                super::InReleaseTarget {
                    url,
                    local_path: PathBuf::from(local_filename),
                    signed_by_key,
                    packages_urls,
                }
            })
            .collect()
    }
}
