pub mod dist;

use std::path::Path;

/// サポートされている主要なパッケージマネージャーの列挙型。
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PackageManager {
    Apt,    // Debian/Ubuntu/Mint など
    Dnf,    // Fedora/RHEL/CentOS 8+ など (dnf.conf)
    Yum,    // RHEL/CentOS 7- など (yum.conf)
    Pacman, // Arch Linux/Manjaro など
    Zypper, // openSUSE など
    Emerge, // Gentoo (Portage)
    Apk,    // Alpine Linux
    None,   // 検出できなかった場合
}

/// Linuxシステムで使用されている主要なパッケージマネージャーを検出します。
/// 検出されたPackageManager enumを返します。
fn detect_package_manager() -> PackageManager {
    // 優先度の高い順にチェックを行います。

    // 1. Debian/Ubuntu 系 (apt)
    if Path::new("/etc/apt/sources.list").exists() {
        return PackageManager::Apt;
    }

    // 2. Fedora/RHEL 系 (dnf/yum)
    if Path::new("/etc/dnf/dnf.conf").exists() {
        return PackageManager::Dnf;
    }
    if Path::new("/etc/yum.conf").exists() {
        return PackageManager::Yum;
    }

    // 3. Arch Linux 系 (pacman)
    if Path::new("/etc/pacman.conf").exists() {
        return PackageManager::Pacman;
    }

    // 4. openSUSE 系 (zypper)
    if Path::new("/etc/zypp/zypp.conf").exists() {
        return PackageManager::Zypper;
    }

    // 5. Gentoo 系 (emerge/portage)
    if Path::new("/etc/portage/make.conf").exists() {
        return PackageManager::Emerge;
    }

    // 6. Alpine Linux (apk)
    if Path::new("/etc/apk/world").exists() {
        return PackageManager::Apk;
    }

    // どのパッケージマネージャーも検出できなかった場合
    PackageManager::None
}
