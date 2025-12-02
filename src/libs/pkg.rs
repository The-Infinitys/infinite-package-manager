use std::fmt::Display;

use crate::{
    libs::system::{self, PackageManager},
    modules::error::UpmError,
};

pub mod deb;

pub enum PackageEntry {
    Deb(deb::DebPackageEntry),
}
impl Display for PackageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deb(deb_package_entry) => write!(f, "{}", deb_package_entry),
        }
    }
}
impl PackageEntry {
    pub fn load() -> Result<Vec<Self>, UpmError> {
        match system::PackageManager::get() {
            PackageManager::Dpkg => {
                let entries = deb::DebPackageEntry::load_all()?;
                // let mut debugs: HashMap<String, u32> = HashMap::new();

                // for entry in entries.iter() {
                //     // extra_fields のイテレーション
                //     for key in entry.extra_fields.keys() {
                //         // エントリのキーの出現回数をインクリメント
                //         *debugs.entry(key.clone()).or_insert(0) += 1;
                //     }
                // }

                // // --- 結果の出力 ---
                // eprintln!("--- Extra Fields Occurrence Count ---");
                // // 出現回数の降順でソートして表示すると、重要な不足フィールドが見つけやすいです
                // let mut sorted_debugs: Vec<(String, u32)> = debugs.into_iter().collect();
                // sorted_debugs.sort_by(|a, b| b.1.cmp(&a.1));

                // for (key, count) in sorted_debugs {
                //     eprintln!("{}: {}", key, count);
                // }
                Ok(entries.into_iter().map(PackageEntry::Deb).collect())
            }
            _ => Err(UpmError::Unsupported),
        }
    }
}

pub fn print_list() -> Result<(), UpmError> {
    let entries = PackageEntry::load()?;
    for entry in entries {
        println!("{}", entry);
    }
    Ok(())
}
