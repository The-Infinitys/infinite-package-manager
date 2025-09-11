/// src/modules/compats/pkg.rs
/// パッケージ関連の互換レイヤー
pub mod dpkg;
pub mod flatpak;
pub mod ipak;
pub mod snap;
use crate::version::{Version, VersionRange};
pub enum PackageType {
    Dpkg,
    Flatpak,
    Snap,
    Ipak,
}

pub struct PackageInfo {
    pub author: AuthorInfo,
    pub name: String,
    pub version: Version,
    pub description: PackageDescriptions,
    pub relation: PackagesRelations,
}
pub struct AuthorInfo {
    pub name: String,
    pub email: String,
}
pub struct PackageDescriptions {
    pub summary: String,
    pub detail: String,
}

pub struct PackagesRelations {
    pub depend_required: Vec<Vec<PackageRange>>,
    pub recommend_suggested: Vec<Vec<PackageRange>>,
    pub break_conflicts: Vec<Vec<PackageRange>>,
    pub provided_virtuals: Vec<PackageVersion>,
}
pub struct PackageRange {
    pub range: VersionRange,
    pub name: String,
}
pub struct PackageVersion {
    pub version: Version,
    pub name: String,
}
