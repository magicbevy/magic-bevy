use magic_log::{mgprint, mgsuccess};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub(crate) const TAG: &str = "EngineVersion";
const MAGIC_TOML: &str = "magic.toml";
const VERSIONS_TOML: &str = "versions.toml";

#[derive(Debug, Deserialize)]
struct MagicToml {
    version: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    channel: String,
    portal: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineReleasesFile {
    #[serde(default)]
    pub releases: Vec<EngineReleaseEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineReleaseEntry {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub channel: String,
    pub portal: String,
}

pub fn bump() -> Result<(), String> {
    let magic_content = fs::read_to_string(MAGIC_TOML)
        .map_err(|e| format!("failed to read {MAGIC_TOML}: {e}"))?;
    let magic: MagicToml = toml::from_str(&magic_content)
        .map_err(|e| format!("failed to parse {MAGIC_TOML}: {e}"))?;

    let mut releases_file = if Path::new(VERSIONS_TOML).exists() {
        let content = fs::read_to_string(VERSIONS_TOML)
            .map_err(|e| format!("failed to read {VERSIONS_TOML}: {e}"))?;
        toml::from_str::<EngineReleasesFile>(&content).unwrap_or(EngineReleasesFile {
            releases: Vec::new(),
        })
    } else {
        EngineReleasesFile { releases: Vec::new() }
    };

    if releases_file
        .releases
        .iter()
        .any(|r| r.version == magic.version)
    {
        mgprint!(TAG, format!("version {} already recorded in {VERSIONS_TOML}", magic.version));
        return Ok(());
    }

    releases_file.releases.push(EngineReleaseEntry {
        version: magic.version.clone(),
        name: Some(magic.name.clone()),
        description: magic.description.clone(),
        channel: magic.channel.clone(),
        portal: magic.portal.clone(),
    });

    let serialized =
        toml::to_string_pretty(&releases_file)
            .map_err(|e| format!("failed to serialize {VERSIONS_TOML}: {e}"))?;

    fs::write(VERSIONS_TOML, serialized)
        .map_err(|e| format!("failed to write {VERSIONS_TOML}: {e}"))?;

    mgsuccess!(
        TAG,
        format!(
            "engine bumped to v{} \"{}\" (channel={}, portal={}) recorded in {VERSIONS_TOML}",
            magic.version, magic.name, magic.channel, magic.portal
        )
    );

    Ok(())
}

