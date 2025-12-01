use super::AptRepositoryEntry;
use super::AptRepositoryType;
use crate::modules::error::UpmError;
use deb822_lossless::Deb822;
use std::path::Path;

pub fn sources(path: impl AsRef<Path>) -> Result<Vec<AptRepositoryEntry>, UpmError> {
    let deb_info = Deb822::from_file(&path)?.paragraphs();
    let mut repo_entries: Vec<AptRepositoryEntry> = vec![];
    for info in deb_info {
        let mut repo_entry = AptRepositoryEntry {
            enabled: true, // デフォルトで有効とする
            ..AptRepositoryEntry::new()
        };
        for entry in info.entries() {
            let key = entry.key();
            let value = entry.value();

            if let Some(key) = key {
                match key.as_str() {
                    "Types" => {
                        // Types の抽出とパース
                        repo_entry.repo_type = value
                            .split_ascii_whitespace()
                            .filter_map(|value| AptRepositoryType::try_from(value).ok())
                            .collect();
                    }
                    "URIs" => {
                        // URIs の抽出
                        repo_entry.uris = value.to_string();
                    }
                    "Suites" => {
                        // Suites の抽出 (スペース区切り)
                        repo_entry.suites = value
                            .split_ascii_whitespace()
                            .map(|s| s.to_string())
                            .collect();
                    }
                    "Components" => {
                        // Components の抽出 (スペース区切り)
                        repo_entry.components = value
                            .split_ascii_whitespace()
                            .map(|s| s.to_string())
                            .collect();
                    }
                    // TODO: Signed-By や Options の処理もここに追加する
                    _ => {
                        repo_entry.options.insert(key, value);
                    }
                }
            }
        }
        // 3. 完全に解析されたエントリを結果に追加
        // URIsが設定されていないエントリは無視するなどのエラーチェックも可能
        if !repo_entry.uris.is_empty() {
            repo_entries.push(repo_entry);
        }
    }

    Ok(repo_entries)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_ubuntu_sources() -> Result<(), UpmError> {
        // 1. **ファイルのパスを作成**
        // `env!("CARGO_MANIFEST_DIR")` は Cargo.toml が存在するディレクトリ (プロジェクトルート) を取得します。
        // それを基点に相対パスを構築するのが、テストでファイルを扱う際の標準的な方法です。
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let test_file_path = format!("{}/tests/apt/ubuntu.sources", manifest_dir);

        println!("Testing file path: {}", test_file_path);

        // 2. **解析関数の呼び出し**
        let entries = sources(test_file_path)?;

        // 3. **アサーション（結果の確認）**

        // エントリが少なくとも1つ読み込まれていること
        assert!(
            !entries.is_empty(),
            "Expected at least one repository entry, but found zero."
        );

        // 読み込まれた最初のエントリの内容を確認
        let first_entry = &entries[0];

        // Type が正しくパースされていること
        assert_eq!(first_entry.repo_type.len(), 1);
        assert!(first_entry.repo_type.contains(&AptRepositoryType::Deb));

        // URIs が正しく読み込まれていること
        assert_eq!(first_entry.uris, "http://archive.ubuntu.com/ubuntu/");

        // Suites が正しく読み込まれていること（ここでは少なくとも3つあるか確認）
        assert!(first_entry.suites.contains(&"noble".to_string()));
        assert_eq!(first_entry.suites.len(), 3);

        // Components が正しく読み込まれていること（ここでは4つあるか確認）
        assert_eq!(first_entry.components.len(), 4);

        // enabled が true であること
        assert!(first_entry.enabled);

        Ok(())
    }
}
