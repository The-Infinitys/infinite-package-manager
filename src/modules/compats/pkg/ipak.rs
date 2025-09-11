//! `ipak`パッケージマネージャーとの互換性レイヤー。
//!
//! このモジュールは、`ipak`固有のデータ構造を、共通の`PackageInfo`構造体に
//! 変換するための機能を提供します。

use crate::modules::compats::pkg::{
    AuthorInfo, PackageDescriptions, PackageInfo, PackageRange, PackageVersion, PackagesRelations,
};

// 互換性レイヤーの対象となる`ipak`のデータ構造をインポート
// `ipak`の型と、現在のクレートの型を区別するために、エイリアスを使用します。
use ::ipak::modules::pkg::{
    AuthorAboutData as IpakAuthorAboutData, PackageAboutData as IpakPackageAboutData,
    PackageData as IpakPackageData, PackageRange as IpakPackageRange,
    PackageVersion as IpakPackageVersion, RelationData as IpakRelationData,
};

/// `ipak`の`PackageData`から、共通の`PackageInfo`への変換を実装します。
impl From<IpakPackageData> for PackageInfo {
    fn from(ipak_data: IpakPackageData) -> Self {
        PackageInfo {
            author: AuthorInfo::from(ipak_data.about.author),
            name: ipak_data.about.package.name.clone(), // 所有権を移動しないようにclone()を使用
            version: ipak_data.about.package.version,
            description: PackageDescriptions {
                summary: ipak_data.about.package.name, // ここで所有権を移動させる
                detail: ipak_data.about.package.description,
            },
            relation: PackagesRelations::from(ipak_data.relation),
        }
    }
}

/// `ipak`の`AuthorAboutData`から、共通の`AuthorInfo`への変換を実装します。
impl From<IpakAuthorAboutData> for AuthorInfo {
    fn from(ipak_author: IpakAuthorAboutData) -> Self {
        AuthorInfo {
            name: ipak_author.name,
            email: ipak_author.email,
        }
    }
}

/// `ipak`の`PackageAboutData`から、共通の`PackageDescriptions`への変換を実装します。
impl From<IpakPackageAboutData> for PackageDescriptions {
    fn from(ipak_pkg: IpakPackageAboutData) -> Self {
        PackageDescriptions {
            summary: ipak_pkg.name,
            detail: ipak_pkg.description,
        }
    }
}

/// `ipak`の`RelationData`から、共通の`PackagesRelations`への変換を実装します。
impl From<IpakRelationData> for PackagesRelations {
    fn from(ipak_rel: IpakRelationData) -> Self {
        let mut recommend_suggested = ipak_rel
            .recommends
            .into_iter()
            .map(|group| group.into_iter().map(PackageRange::from).collect())
            .collect::<Vec<Vec<PackageRange>>>();

        let suggests: Vec<Vec<PackageRange>> = ipak_rel
            .suggests
            .into_iter()
            .map(|group| group.into_iter().map(PackageRange::from).collect())
            .collect();

        recommend_suggested.extend(suggests); // concatの代わりにextendを使用

        PackagesRelations {
            depend_required: ipak_rel
                .depend
                .into_iter()
                .map(|group| group.into_iter().map(PackageRange::from).collect())
                .collect(),
            recommend_suggested, // 結合したベクターを使用
            break_conflicts: ipak_rel
                .conflicts
                .into_iter()
                .map(|c| vec![PackageRange::from(c)])
                .collect(),
            provided_virtuals: ipak_rel
                .virtuals
                .into_iter()
                .map(PackageVersion::from)
                .collect(),
        }
    }
}

/// `ipak`の`PackageRange`から、共通の`PackageRange`への変換を実装します。
impl From<IpakPackageRange> for PackageRange {
    fn from(ipak_range: IpakPackageRange) -> Self {
        PackageRange {
            name: ipak_range.name,
            range: ipak_range.range,
        }
    }
}

/// `ipak`の`PackageVersion`から、共通の`PackageVersion`への変換を実装します。
impl From<IpakPackageVersion> for PackageVersion {
    fn from(ipak_version: IpakPackageVersion) -> Self {
        PackageVersion {
            name: ipak_version.name,
            version: ipak_version.version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::{Version, VersionRange};
    use ::ipak::modules::pkg::{
        AboutData as IpakAboutData, AuthorAboutData as IpakAuthorAboutData,
        PackageAboutData as IpakPackageAboutData, PackageData as IpakPackageData,
        PackageRange as IpakPackageRange, PackageVersion as IpakPackageVersion,
        RelationData as IpakRelationData,
    };
    use std::str::FromStr;

    #[test]
    fn test_package_data_to_package_info_conversion() {
        // テスト用のIpakPackageDataを作成
        let ipak_data = IpakPackageData {
            about: IpakAboutData {
                author: IpakAuthorAboutData {
                    name: "Test Author".to_string(),
                    email: "test@example.com".to_string(),
                },
                package: IpakPackageAboutData {
                    name: "test-package".to_string(),
                    version: Version::from_str("1.2.3").unwrap(),
                    description: "This is a test package.".to_string(),
                },
            },
            relation: IpakRelationData {
                depend: vec![vec![IpakPackageRange {
                    name: "dep-a".to_string(),
                    range: VersionRange::from_str(">= 1.0").unwrap(),
                }]],
                recommends: vec![vec![IpakPackageRange {
                    name: "rec-b".to_string(),
                    range: VersionRange::from_str("= 2.0").unwrap(),
                }]],
                suggests: vec![vec![IpakPackageRange {
                    name: "sugg-c".to_string(),
                    range: VersionRange::from_str("< 3.0").unwrap(),
                }]],
                conflicts: vec![IpakPackageRange {
                    name: "conf-d".to_string(),
                    range: VersionRange::from_str("0.5").unwrap(),
                }],
                virtuals: vec![IpakPackageVersion {
                    name: "virtual-pkg".to_string(),
                    version: Version::from_str("1.0.0").unwrap(),
                }],
                ..Default::default()
            },
            ..Default::default()
        };

        // Fromトレイトを使って変換
        let pkg_info: PackageInfo = ipak_data.into();

        // 変換結果が正しいことを確認
        assert_eq!(pkg_info.name, "test-package");
        assert_eq!(pkg_info.version, Version::from_str("1.2.3").unwrap());
        assert_eq!(pkg_info.author.name, "Test Author");
        assert_eq!(pkg_info.author.email, "test@example.com");
        assert_eq!(pkg_info.description.summary, "test-package");
        assert_eq!(pkg_info.description.detail, "This is a test package.");

        // 依存関係の確認
        assert_eq!(pkg_info.relation.depend_required.len(), 1);
        assert_eq!(pkg_info.relation.depend_required[0][0].name, "dep-a");
        assert_eq!(
            pkg_info.relation.depend_required[0][0].range.to_string(),
            ">= 1.0"
        );

        // 推奨・提案の結合確認
        assert_eq!(pkg_info.relation.recommend_suggested.len(), 2);
        assert_eq!(pkg_info.relation.recommend_suggested[0][0].name, "rec-b");
        assert_eq!(pkg_info.relation.recommend_suggested[1][0].name, "sugg-c");

        // 競合関係の確認
        assert_eq!(pkg_info.relation.break_conflicts.len(), 1);
        assert_eq!(pkg_info.relation.break_conflicts[0][0].name, "conf-d");

        // 仮想パッケージの確認
        assert_eq!(pkg_info.relation.provided_virtuals.len(), 1);
        assert_eq!(pkg_info.relation.provided_virtuals[0].name, "virtual-pkg");
    }

    #[test]
    fn test_empty_relation_conversion() {
        let ipak_data = IpakPackageData::default();
        let pkg_info: PackageInfo = ipak_data.into();

        assert!(pkg_info.relation.depend_required.is_empty());
        assert!(pkg_info.relation.recommend_suggested.is_empty());
        assert!(pkg_info.relation.break_conflicts.is_empty());
        assert!(pkg_info.relation.provided_virtuals.is_empty());
    }
}
