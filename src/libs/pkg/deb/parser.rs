use super::DebPackageEntry;
use crate::modules::error::UpmError;
use deb822_lossless::Deb822;
use std::path::Path;

pub fn parse_deb_status_file(path: impl AsRef<Path>) -> Result<Vec<DebPackageEntry>, UpmError> {
    let deb_info = Deb822::from_file(&path)?.paragraphs();
    let mut package_entries: Vec<DebPackageEntry> = vec![];

    for info in deb_info {
        let mut package_entry = DebPackageEntry::new();
        for entry in info.entries() {
            let key = entry.key();
            let value = entry.value();

            if let Some(key) = key {
                match key.as_str() {
                    "Package" => package_entry.package = value.to_string(),
                    "Version" => package_entry.version = value.to_string(),
                    "Architecture" => package_entry.architecture = value.to_string(),
                    "Description" => package_entry.description = value.to_string(),
                    "Installed-Size" => {
                        package_entry.installed_size = value.parse().unwrap_or(0)
                    }
                    "Maintainer" => package_entry.maintainer = value.to_string(),
                    "Homepage" => package_entry.homepage = Some(value.to_string()),
                    "Depends" => {
                        package_entry.depends = value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    }
                    "Status" => package_entry.status = value.to_string(),
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

    #[test]
    fn test_parse_deb_status_file() -> Result<(), UpmError> {
        let content = include_str!("../../../../tests/dpkg/apt-control");

        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", content)?;

        let entries = parse_deb_status_file(temp_file.path())?;

        assert_eq!(entries.len(), 2);

        // apt パッケージの検証
        let apt_package = &entries[0];
        assert_eq!(apt_package.package, "apt");
        assert_eq!(apt_package.version, "2.7.7");
        assert_eq!(apt_package.architecture, "amd64");
        assert!(apt_package.description.starts_with("commandline package manager"));
        assert_eq!(apt_package.installed_size, 5293);
        assert_eq!(apt_package.maintainer, "Debian APT Team <deity@lists.debian.org>");
        assert_eq!(apt_package.homepage, Some("https://wiki.debian.org/Apt".to_string()));
        assert_eq!(apt_package.depends, vec!["some-dependency (>= 1.0)".to_string()]);
        assert_eq!(apt_package.status, "install ok installed");
        assert!(apt_package.extra_fields.contains_key("Original-Maintainer"));

        // bash パッケージの検証
        let bash_package = &entries[1];
        assert_eq!(bash_package.package, "bash");
        assert_eq!(bash_package.version, "5.2.15-2");
        assert_eq!(bash_package.architecture, "amd64");
        assert!(bash_package.description.starts_with("GNU Bourne Again SHell"));
        assert_eq!(bash_package.installed_size, 1729);
        assert_eq!(bash_package.maintainer, "Debian Bash Maintainers <pkg-bash-devel@lists.alioth.debian.org>");
        assert_eq!(bash_package.homepage, None); // bash には homepage がない
        assert!(bash_package.depends.is_empty());
        assert_eq!(bash_package.status, "install ok installed");
        assert!(bash_package.extra_fields.is_empty());

        Ok(())
    }

    #[test]
    fn test_parse_dpkg_s_output() -> Result<(), UpmError> {
        let content = include_str!("../../../../tests/dpkg/deb-control");

        let entry = DebPackageEntry::load_from_str(content)?;

        assert_eq!(entry.package, "git");
        assert_eq!(entry.status, "install ok installed");
        assert_eq!(entry.architecture, "amd64");
        assert_eq!(entry.version, "1:2.51.0-1ubuntu1");
        assert_eq!(entry.installed_size, 25960);
        assert_eq!(entry.maintainer, "Ubuntu Developers <ubuntu-devel-discuss @lists.ubuntu.com>");
        assert_eq!(entry.homepage, Some("https://git-scm.com/".to_string()));
        assert!(entry.description.starts_with("fast, scalable, distributed revision control system"));
        assert!(entry.depends.len() > 5); // 少なくともいくつかの依存関係があることを確認

        assert!(entry.extra_fields.contains_key("Priority"));
        assert_eq!(entry.extra_fields["Priority"], "optional");
        assert!(entry.extra_fields.contains_key("Section"));
        assert_eq!(entry.extra_fields["Section"], "vcs");
        assert!(entry.extra_fields.contains_key("Multi-Arch"));
        assert_eq!(entry.extra_fields["Multi-Arch"], "foreign");

        Ok(())
    }
}

