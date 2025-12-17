use deb822_lossless::Deb822;
use flate2::read::GzDecoder;
use lzma_rs::lzma_decompress;
use reqwest;
use std::io::Read;
use xz2::read::XzDecoder;

use crate::{libs::pkg::deb::DebPackageEntry, modules::error::Error};

pub async fn parse_packages_file(url: &str) -> Result<Vec<DebPackageEntry>, Error> {
    let file_name = url
        .rsplit_once('/')
        .map(|(_, suffix)| suffix)
        .unwrap_or(url)
        .to_string();
    let response = reqwest::get(url).await?.bytes().await?.to_vec();
    let mut decompressed_data = Vec::new();
    let reader: Box<dyn Read> = if file_name.ends_with(".gz") {
        Box::new(GzDecoder::new(response.as_slice()))
    } else if file_name.ends_with(".xz") {
        Box::new(XzDecoder::new(response.as_slice()))
    } else if file_name.ends_with(".lzma") {
        lzma_decompress(&mut response.as_slice(), &mut decompressed_data)?;
        Box::new(decompressed_data.as_slice())
    } else {
        Box::new(response.as_slice())
    };

    let deb_info = Deb822::read(reader)?.paragraphs();
    let mut package_entries: Vec<DebPackageEntry> = vec![];

    for info in deb_info {
        let mut package_entry = DebPackageEntry::new();
        for entry in info.entries() {
            let key = entry.key();
            let value = entry.value();

            if let Some(key) = key {
                // This parsing logic is a duplicate of `src/libs/pkg/deb/parser.rs`
                // Consider refactoring to share this logic.
                match key.as_str() {
                    "Package" => package_entry.package = value.to_string(),
                    "Version" => package_entry.version = value.to_string(),
                    "Architecture" => package_entry.architecture = value.to_string(),
                    "Description" => package_entry.description = value.to_string(),
                    "Installed-Size" => package_entry.installed_size = value.parse().unwrap_or(0),
                    "Maintainer" => package_entry.maintainer = value.to_string(),
                    "Homepage" => package_entry.homepage = Some(value.to_string()),
                    "Depends" => package_entry.depends = parse_comma_separated_list(&value),
                    "Status" => package_entry.status = value.to_string(),
                    "Priority" => package_entry.priority = value.to_string(),
                    "Section" => package_entry.section = value.to_string(),
                    "Source" => package_entry.source = Some(value.to_string()),
                    "Replaces" => package_entry.replaces = parse_comma_separated_list(&value),
                    "Provides" => package_entry.provides = parse_comma_separated_list(&value),
                    "Conflicts" => package_entry.conflicts = parse_comma_separated_list(&value),
                    "Pre-Depends" => package_entry.pre_depends = parse_comma_separated_list(&value),
                    "Breaks" => package_entry.breaks = parse_comma_separated_list(&value),
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
                    "Enhances" => package_entry.enhances = parse_comma_separated_list(&value),
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
        if !package_entry.package.is_empty() {
            package_entries.push(package_entry);
        }
    }
    Ok(package_entries)
}

// Helper function duplicated from `src/libs/pkg/deb/parser.rs`
fn parse_comma_separated_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
