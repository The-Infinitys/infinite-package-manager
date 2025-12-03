use std::collections::HashSet;
use super::AptRepositoryEntry;

/// AptRepositoryEntryのベクタに対する拡張トレイト
pub trait AptRepositoryEntryVec {
    /// 各リポジトリエントリから、InReleaseファイルをダウンロードするためのターゲット情報リストを生成する
    /// 
    /// 戻り値: Vec<(ダウンロードURL: String, ローカル保存パスのファイル名: String)>
    fn in_release_targets(&self) -> Vec<(String, String)>;
}

// AptRepositoryEntryのベクタ（Vec<AptRepositoryEntry>）にトレイトを実装
impl AptRepositoryEntryVec for Vec<AptRepositoryEntry> {
    fn in_release_targets(&self) -> Vec<(String, String)> {
        // Base64エンジンは使用しないが、他の箇所で使う可能性を考慮して残しておく
        let _base64_engine = base64::engine::general_purpose::STANDARD;
        
        // 重複を避けるためにHashSetを使用
        let mut unique_urls = HashSet::new();

        self.iter()
            // 1. 有効なエントリのみを対象とする
            .filter(|entry| entry.enabled)
            // 2. 各エントリを、複数のSuiteに対応するInRelease URLのストリームに展開する
            .for_each(|entry| {
                entry.suites.iter().for_each(|suite| {
                    // InReleaseファイルの標準的なパス形式: URI/dists/SUITE/InRelease
                    let url = format!("{}/dists/{}/InRelease", entry.uris, suite);
                    unique_urls.insert(url);
                });
            });

        // 3. 一意のURLに対して、ローカル保存ファイル名を生成する
        unique_urls.into_iter().map(|url| {
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

            (url, local_filename)
        }).collect()
    }
}