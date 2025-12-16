use super::AptRepositoryEntry;
use super::AptRepositoryType;
use crate::libs::repo::apt::AptRepositoryKeyInfo;
use crate::modules::error::Error;
use base64::Engine;
use deb822_lossless::Deb822;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::path::PathBuf;
/// sources.list形式のファイルからリポジトリエントリを解析します。
pub fn list(path: impl AsRef<Path>) -> Result<Vec<AptRepositoryEntry>, Error> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut repo_entries: Vec<AptRepositoryEntry> = vec![];

    for line in reader.lines() {
        let line = line?;
        // コメント（#以降）を削除し、行頭行末の空白を削除
        let trimmed_line = match line.split_once('#') {
            Some((line, _comment)) => line,
            None => line.as_str(),
        }
        .trim();

        // 短すぎる行はスキップ
        if trimmed_line.len() < 4 {
            continue;
        }

        let mut repo_entry = AptRepositoryEntry::new();
        repo_entry.enabled = true; // sources.list形式は基本的に有効

        // 1. リポジトリタイプ (deb/deb-src) を抽出
        let (repo_type_str, mut remaining) = trimmed_line
            .split_once(char::is_whitespace)
            .unwrap_or((trimmed_line, ""));

        let repo_type = match AptRepositoryType::try_from(repo_type_str) {
            Ok(t) => t,
            Err(_) => continue, // 不正なタイプの場合はスキップ
        };
        repo_entry.repo_types.push(repo_type);

        remaining = remaining.trim_start();

        // 2. インラインオプションブロック ([...]) を抽出
        if remaining.starts_with('[')
            && let Some(end_index) = remaining.find(']')
        {
            // オプションブロック全体を抽出 (e.g., "[signed-by=/keyring.gpg arch=amd64]")
            let options_block = &remaining[1..end_index];

            // オプションをパース
            for option_pair in options_block.split_ascii_whitespace() {
                if let Some((key, value)) = option_pair.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();

                    match key.to_lowercase().as_str() {
                        "signed-by" => {
                            // signed-byはPathとして処理
                            repo_entry.signed_by = AptRepositoryKeyInfo::Path(PathBuf::from(value));
                        }
                        "arch" | "architectures" => {
                            // Architecturesを処理
                            repo_entry.architectures =
                                value.split(',').map(|s| s.trim().to_string()).collect();
                        }
                        _ => {
                            repo_entry
                                .options
                                .insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }

            // オプションブロックとそれに続く空白を、残りの行から削除
            remaining = remaining[(end_index + 1)..].trim_start();
        }

        // 3. URI, Suite, Components を抽出
        let entries: Vec<&str> = remaining.split_ascii_whitespace().collect();

        // 必須フィールド (URI, Suite) があるかチェック
        if entries.len() < 2 {
            // URIまたはSuiteがない場合はスキップ（エラーにせず）
            continue;
        }

        // URI
        repo_entry.uris = entries[0].to_string();

        // Suite
        repo_entry.suites.push(entries[1].to_string());

        // Components (残りのエントリ)
        for entry in entries.into_iter().skip(2) {
            repo_entry.components.push(entry.to_string());
        }

        // URIsが設定されているエントリのみを追加
        if !repo_entry.uris.is_empty() {
            repo_entries.push(repo_entry);
        }
    }

    Ok(repo_entries)
}

pub fn sources(path: impl AsRef<Path>) -> Result<Vec<AptRepositoryEntry>, Error> {
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
                        repo_entry.repo_types = value
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
                    "Signed-By" => {
                        let value = value.trim();
                        let begin_pgp = "-----BEGIN PGP PUBLIC KEY BLOCK-----";
                        let end_pgp = "-----END PGP PUBLIC KEY BLOCK-----";
                        let trim_value = value
                            .strip_prefix(begin_pgp)
                            .and_then(|v| v.strip_suffix(end_pgp));
                        repo_entry.signed_by = match trim_value {
                            Some(value) => {
                                let bin = value
                                    .split("\n")
                                    .map(|s| s.trim())
                                    .filter(|s| s != &".")
                                    .collect::<Vec<&str>>()
                                    .join("");
                                let b = &base64::engine::general_purpose::STANDARD;
                                let bin = b.decode(bin)?;
                                AptRepositoryKeyInfo::Bin(bin)
                            }
                            None => AptRepositoryKeyInfo::Path(PathBuf::from(value)),
                        }
                    }
                    "Architectures" => {
                        // Architecturesはスペース区切りのリスト
                        repo_entry.architectures = value
                            .split_ascii_whitespace()
                            .map(|s| s.to_string())
                            .collect();
                    }
                    _ => {
                        repo_entry
                            .options
                            .insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        // URIsが設定されているエントリのみを追加
        if !repo_entry.uris.is_empty() {
            repo_entries.push(repo_entry);
        }
    }

    Ok(repo_entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    // list 形式のテスト用関数
    #[test]
    fn test_read_sources_list() -> Result<(), Error> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let test_file_path = format!("{}/tests/apt/debian.list", manifest_dir);
        println!("Testing file path: {}", test_file_path);
        let entries = list(test_file_path)?;
        assert!(
            entries.len() >= 4,
            "Expected at least four repository entries, but found {}.",
            entries.len()
        );
        let first_entry = &entries[0];
        println!("{:#?}", first_entry);
        assert_eq!(first_entry.repo_types.len(), 1);
        assert_eq!(first_entry.repo_types, vec![AptRepositoryType::Deb]);
        assert_eq!(first_entry.uris, "http://deb.debian.org/debian".to_string());
        assert!(first_entry.suites.contains(&"bookworm".to_string()));
        assert_eq!(first_entry.components.len(), 2);
        assert!(first_entry.enabled);
        Ok(())
    }
    #[test]
    fn test_read_ubuntu_sources() -> Result<(), Error> {
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
        assert_eq!(first_entry.repo_types.len(), 2);
        assert_eq!(
            first_entry.repo_types,
            vec![AptRepositoryType::Deb, AptRepositoryType::DebSrc]
        );

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
