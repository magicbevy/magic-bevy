use crate::error::PackageError;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageTarget {
    Editor,
    Runtime,
}

impl PackageTarget {
    pub fn directory(self) -> &'static str {
        match self {
            Self::Editor => "editor",
            Self::Runtime => "runtime",
        }
    }

    pub fn manifest_value(self) -> &'static str {
        match self {
            Self::Editor => "editor",
            Self::Runtime => "runtime",
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RawManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub vendor: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub magic: Option<String>,
    #[serde(default = "default_channel")]
    pub channel: String, // (stable, preview, etc.)
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
}

fn default_channel() -> String {
    "stable".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub category: String,
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub targets: BTreeSet<PackageTarget>,
    pub root: PathBuf,
}

impl PackageManifest {
    pub fn id(&self) -> String {
        format!("{}/{}", self.vendor, self.name)
    }

    pub fn dependency_identifier(&self) -> String {
        format!(
            "{}_{}",
            crate::utils::cargo_identifier(&self.vendor),
            crate::utils::cargo_identifier(&self.name)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProjectPackagesManifest {
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
}

impl ProjectPackagesManifest {
    pub fn load_from_file(path: &Path) -> Result<Self, PackageError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path).map_err(|source| PackageError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let manifest = toml::from_str(&content).map_err(|source| PackageError::Manifest {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(manifest)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), PackageError> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = toml::to_string_pretty(self).map_err(|source| PackageError::InvalidManifest {
            path: path.to_path_buf(),
            message: format!("Failed to serialize TOML: {source}"),
        })?;
        std::fs::write(path, content).map_err(|source| PackageError::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PackageReleasesFile {
    pub package: ReleasePackageMeta,
    #[serde(default)]
    pub releases: Vec<ReleaseEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleasePackageMeta {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReleaseEntry {
    pub version: String,
    pub magic: String,
    pub timestamp: u64,
    #[serde(default = "default_channel")]
    pub channel: String,
}