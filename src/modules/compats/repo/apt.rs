/// src/modules/compats/repo/apt.rs
/// aptレポジトリの互換レイヤー
use crate::Error;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoInfo {
    pub architectures: Vec<String>,
    pub types: Vec<String>,
    pub uris: Vec<String>,
    pub suites: Vec<String>,
    pub components: Vec<String>,
    pub signed_by: Option<Signature>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub path: Option<PathBuf>,
    pub data: Vec<u8>,
}

pub fn parse_repo_info(content: &str) -> Result<Vec<RepoInfo>, Error> {
    let mut repos: Vec<RepoInfo> = Vec::new();
    let mut current_repo: Option<RepoInfo> = None;
    let mut last_key: Option<String> = None;

    for line in content.lines() {
        let trimmed_line = line.trim_start();

        if trimmed_line.is_empty() {
            // Blank line signifies the end of a stanza
            if let Some(repo) = current_repo.take() {
                repos.push(repo);
            }
            last_key = None; // Reset last_key after a blank line
            continue;
        }

        if trimmed_line.starts_with('#') {
            // Skip comments
            continue;
        }

        if line.starts_with(char::is_whitespace) {
            // This is a continuation line
            if let Some(key) = &last_key {
                if let Some(repo) = current_repo.as_mut() {
                    match key.as_str() {
                        "Types" => repo
                            .types
                            .extend(trimmed_line.split_whitespace().map(|s| s.to_string())),
                        "URIs" => repo
                            .uris
                            .extend(trimmed_line.split_whitespace().map(|s| s.to_string())),
                        "Suites" => repo
                            .suites
                            .extend(trimmed_line.split_whitespace().map(|s| s.to_string())),
                        "Components" => repo
                            .components
                            .extend(trimmed_line.split_whitespace().map(|s| s.to_string())),
                        "Architectures" => repo
                            .architectures
                            .extend(trimmed_line.split_whitespace().map(|s| s.to_string())),
                        _ => {
                            // Ignore unknown keys for now
                        }
                    }
                }
            } else {
                return Err(Error::ParseError(format!(
                    "Indented line without a preceding key: {}",
                    line
                )));
            }
        } else {
            // This is a new key-value pair, non-indented
            if current_repo.is_none() {
                // Start a new RepoInfo if not already in one
                current_repo = Some(RepoInfo {
                    architectures: Vec::new(),
                    types: Vec::new(),
                    uris: Vec::new(),
                    suites: Vec::new(),
                    components: Vec::new(),
                    signed_by: None,
                });
            }

            if let Some((key, value)) = trimmed_line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                last_key = Some(key.to_string());

                let repo = current_repo.as_mut().unwrap(); // We know current_repo is Some here

                match key {
                    "Types" => {
                        repo.types = value.split_whitespace().map(|s| s.to_string()).collect()
                    }
                    "URIs" => repo.uris = value.split_whitespace().map(|s| s.to_string()).collect(),
                    "Suites" => {
                        repo.suites = value.split_whitespace().map(|s| s.to_string()).collect()
                    }
                    "Components" => {
                        repo.components = value.split_whitespace().map(|s| s.to_string()).collect()
                    }
                    "Signed-By" => {
                        let path = value;
                        let data = fs::read(path).unwrap_or_default();
                        repo.signed_by = Some(Signature {
                            path: Some(PathBuf::from(path)),
                            data,
                        });
                    }
                    "Architectures" => {
                        repo.architectures =
                            value.split_whitespace().map(|s| s.to_string()).collect()
                    }
                    _ => {
                        // Ignore unknown keys for now
                    }
                }
            } else {
                return Err(Error::ParseError(format!(
                    "Invalid key-value pair format: {}",
                    line
                )));
            }
        }
    }

    // Push the last repo if it exists
    if let Some(repo) = current_repo {
        repos.push(repo);
    }

    Ok(repos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const TEST_SOURCES_CONTENT: &str = include_str!("tests/ubuntu.sources");

    #[test]
    fn test_parse_repo_info() {
        let repos = parse_repo_info(TEST_SOURCES_CONTENT).unwrap();

        assert_eq!(repos.len(), 2);

        // Test first repo
        let repo1 = &repos[0];
        assert_eq!(repo1.types, vec!["deb", "deb-src"]);
        assert_eq!(repo1.uris, vec!["https://archive.ubuntu.com/ubuntu"]);
        assert_eq!(
            repo1.suites,
            vec!["plucky", "plucky-updates", "plucky-backports"]
        );
        assert_eq!(
            repo1.components,
            vec!["main", "universe", "restricted", "multiverse"]
        );
        // For signed_by, we now compare the path of the file
        let expected_signed_by_path =
            PathBuf::from("/usr/share/keyrings/ubuntu-archive-keyring.gpg");
        assert_eq!(
            repo1.signed_by.as_ref().unwrap().path.as_ref().unwrap(),
            &expected_signed_by_path
        );
        assert!(repo1.architectures.is_empty()); // Architectures not specified in this block

        // Test second repo
        let repo2 = &repos[1];
        assert_eq!(repo2.types, vec!["deb", "deb-src"]);
        assert_eq!(repo2.uris, vec!["https://security.ubuntu.com/ubuntu/"]);
        assert_eq!(repo2.suites, vec!["plucky-security"]);
        assert_eq!(
            repo2.components,
            vec!["main", "universe", "restricted", "multiverse"]
        );
        // For signed_by, we now compare the path of the file
        let expected_signed_by_path =
            PathBuf::from("/usr/share/keyrings/ubuntu-archive-keyring.gpg");
        assert_eq!(
            repo2.signed_by.as_ref().unwrap().path.as_ref().unwrap(),
            &expected_signed_by_path
        );
        assert!(repo2.architectures.is_empty()); // Architectures not specified in this block
    }
}
