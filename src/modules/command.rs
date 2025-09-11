mod license;
mod manual;
mod package;
mod repo;
pub use license::run as license;
pub use manual::run as manual;
pub use package::run as pkg;
pub use repo::run as repo;
