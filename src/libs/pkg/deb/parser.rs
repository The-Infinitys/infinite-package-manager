use super::DebPackageEntry;
use crate::modules::error::UpmError;
use deb822_lossless::Deb822;
use std::path::Path;

/// カンマ区切りの文字列をトリミングしてVec<String>にパースするヘルパー関数
fn parse_comma_separated_list(value: String) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()) // 空の要素を除去
        .collect()
}

pub fn parse_deb_status_file(path: impl AsRef<Path>) -> Result<Vec<DebPackageEntry>, UpmError> {
    let deb_info = Deb822::from_file(&path)?.paragraphs();
    let mut package_entries: Vec<DebPackageEntry> = vec![];
    for info in deb_info {
        let mut package_entry = DebPackageEntry::new();
        for entry in info.entries() {
            let key = entry.key();
            let value = entry.value();

            if let Some(key) = key {
                // キーを全て小文字にせず、Debianポリシーで規定された大文字小文字のパターンでマッチ
                match key.as_str() {
                    // 基本情報
                    "Package" => package_entry.package = value.to_string(),
                    "Version" => package_entry.version = value.to_string(),
                    "Architecture" => package_entry.architecture = value.to_string(),
                    "Description" => package_entry.description = value.to_string(),
                    "Installed-Size" => package_entry.installed_size = value.parse().unwrap_or(0),
                    "Maintainer" => package_entry.maintainer = value.to_string(),
                    "Homepage" => package_entry.homepage = Some(value.to_string()),
                    "Status" => package_entry.status = value.to_string(),

                    // 新しく追加された単一値フィールド
                    "Priority" => package_entry.priority = value.to_string(),
                    "Section" => package_entry.section = value.to_string(),
                    "Source" => package_entry.source = Some(value.to_string()),

                    // 依存関係フィールド (カンマ区切り)
                    "Depends" => package_entry.depends = parse_comma_separated_list(value),
                    "Replaces" => package_entry.replaces = parse_comma_separated_list(value),
                    "Provides" => package_entry.provides = parse_comma_separated_list(value),
                    "Conflicts" => package_entry.conflicts = parse_comma_separated_list(value),
                    "Pre-Depends" => package_entry.pre_depends = parse_comma_separated_list(value),
                    "Breaks" => package_entry.breaks = parse_comma_separated_list(value),
                    "Recommends" => package_entry.recommends = parse_comma_separated_list(value),
                    "Suggests" => package_entry.suggests = parse_comma_separated_list(value),
                    "Conffiles" => {
                        let files: Vec<String> = value
                            .lines()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        package_entry.conffiles = if files.is_empty() { None } else { Some(files) };
                    }
                    "Original-Maintainer" => {
                        package_entry.original_maintainer = Some(value.to_string())
                    }
                    "Enhances" => package_entry.enhances = parse_comma_separated_list(value), // Dependsと同様にカンマ区切り
                    "Essential" => package_entry.essential = Some(value.to_string()),
                    "Multi-Arch" => package_entry.multi_arch = Some(value.to_string()),
                    _ => {
                        package_entry
                            .extra_fields
                            .insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        // Package名が設定されているエントリのみを追加
        if !package_entry.package.is_empty() {
            package_entries.push(package_entry);
        }
    }
    Ok(package_entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ヘルパー関数はテストから利用できないため、テストケース内で使用する依存関係リストのヘルパーを定義
    fn get_dummy_dependency_list() -> Vec<String> {
        vec!["some-dependency (>= 1.0)".to_string()]
    }

    #[test]
    fn test_parse_deb_status_file() -> Result<(), UpmError> {
        // テスト用のダミーapt-controlファイルをシミュレート
        let content = include_str!("../../../../tests/apt/control");
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", content)?;

        let entries = parse_deb_status_file(temp_file.path())?;

        assert_eq!(entries.len(), 2);

        // apt パッケージの検証 (新しいフィールドをチェック)
        let apt_package = &entries[0];
        assert_eq!(apt_package.package, "apt");
        assert_eq!(apt_package.priority, "important"); // 新しいフィールド
        assert_eq!(apt_package.section, "base"); // 新しいフィールド
        assert_eq!(apt_package.source, Some("apt-source (2.7.7)".to_string())); // 新しいフィールド
        assert_eq!(apt_package.depends, get_dummy_dependency_list());
        assert_eq!(
            apt_package.original_maintainer,
            Some("APT Development Team <deity@lists.debian.org>".to_string())
        );
        let bash_package = &entries[1];
        assert_eq!(bash_package.package, "bash");
        assert!(bash_package.source.is_none());
        assert!(bash_package.depends.is_empty());
        assert!(bash_package.extra_fields.is_empty());

        Ok(())
    }
}
