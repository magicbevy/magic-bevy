mod error;
mod manifest;
mod manager;
pub mod utils;

pub use error::PackageError;
pub use manifest::{PackageManifest, PackageTarget, PackageReleasesFile, ReleaseEntry};
pub use manager::{PackageManager, SyncReport};