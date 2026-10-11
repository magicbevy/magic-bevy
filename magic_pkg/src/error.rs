use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum PackageError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Manifest {
        path: PathBuf,
        source: toml::de::Error,
    },
    InvalidManifest {
        path: PathBuf,
        message: String,
    },
    DuplicateId(String),
    AlreadyExists(PathBuf),
    InvalidArgument(String),
    DownloadError(String),
    ZipError(String),
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "I/O error at {}: {source}", path.display()),
            Self::Manifest { path, source } => write!(
                f,
                "Invalid package manifest at {}: {source}",
                path.display()
            ),
            Self::InvalidManifest { path, message } => write!(
                f,
                "Invalid package manifest at {}: {message}",
                path.display()
            ),
            Self::DuplicateId(id) => write!(f, "Duplicate package identifier: {id}"),
            Self::AlreadyExists(path) => write!(f, "Package already exists at {}", path.display()),
            Self::InvalidArgument(message) => write!(f, "Invalid argument: {message}"),
            Self::DownloadError(message) => write!(f, "Download error: {message}"),
            Self::ZipError(message) => write!(f, "Zip extract error: {message}"),
        }
    }
}

impl std::error::Error for PackageError {}