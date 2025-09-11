/// src/modules/compats/pkg.rs
/// パッケージ関連の互換レイヤー
///
/// このモジュールは、複数のパッケージ形式（dpkg, flatpak, snapなど）を
/// 統一的に扱うための抽象化レイヤーを提供します。
pub mod dpkg;
pub mod ipak;

use crate::version::{Version, VersionRange};

/// パッケージのタイプを列挙する`enum`
///
/// 現在サポートしているパッケージ形式を定義します。
pub enum PackageType {
    Dpkg,
    Flatpak,
    Snap,
    Ipak,
}

/// パッケージのメタ情報を保持する`struct`
///
/// パッケージに関する基本的な情報をまとめています。
pub struct PackageInfo {
    /// パッケージの作成者情報
    pub author: AuthorInfo,
    /// パッケージ名
    pub name: String,
    /// パッケージのバージョン
    pub version: Version,
    /// パッケージの説明情報
    pub description: PackageDescriptions,
    /// 他のパッケージとの依存関係
    pub relation: PackagesRelations,
}

/// パッケージ作成者の情報を保持する`struct`
pub struct AuthorInfo {
    /// 作成者の名前
    pub name: String,
    /// 作成者のメールアドレス
    pub email: String,
}

/// パッケージの説明情報を保持する`struct`
pub struct PackageDescriptions {
    /// 簡潔な要約
    pub summary: String,
    /// 詳細な説明
    pub detail: String,
}

/// パッケージ間の依存関係を保持する`struct`
///
/// 必須、推奨、競合といった様々な関係を定義します。
pub struct PackagesRelations {
    /// 必須の依存関係
    ///
    /// `Vec<Vec<PackageRange>>` の形式で、複数の選択肢（OR関係）を持つ依存関係（AND関係）を表現します。
    pub depend_required: Vec<Vec<PackageRange>>,
    /// 推奨・提案されるパッケージ
    pub recommend_suggested: Vec<Vec<PackageRange>>,
    /// 競合・衝突するパッケージ
    pub break_conflicts: Vec<Vec<PackageRange>>,
    /// このパッケージが提供する仮想パッケージ
    pub provided_virtuals: Vec<PackageVersion>,
}

/// パッケージ名とバージョンの範囲を保持する`struct`
///
/// 依存関係のバージョン指定に使用されます。
pub struct PackageRange {
    /// バージョンの範囲
    pub range: VersionRange,
    /// パッケージ名
    pub name: String,
}

/// パッケージ名と特定のバージョンを保持する`struct`
///
/// 仮想パッケージの定義などに使用されます。
pub struct PackageVersion {
    /// パッケージのバージョン
    pub version: Version,
    /// パッケージ名
    pub name: String,
}
