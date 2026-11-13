use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTarget {
    Editor,
    Runtime,
}

impl BuildTarget {
    pub const ALL: [Self; 2] = [Self::Editor, Self::Runtime];

    pub fn name(self) -> &'static str {
        match self {
            Self::Editor => "editor",
            Self::Runtime => "runtime",
        }
    }

    pub fn package(self) -> &'static str {
        match self {
            Self::Editor => "magic_editor",
            Self::Runtime => "magic_runtime",
        }
    }

    pub fn library_name(self) -> String {
        let prefix = std::env::consts::DLL_PREFIX;
        let suffix = std::env::consts::DLL_SUFFIX;
        match self {
            Self::Editor => format!("{prefix}magic_editor{suffix}"),
            Self::Runtime => format!("{prefix}magic_runtime{suffix}"),
        }
    }

    pub fn cache_dir(self) -> PathBuf {
        if separate_targets_enabled() {
            project_root()
                .join(".magic")
                .join("cache")
                .join(self.name())
        } else {
            project_root().join("target")
        }
    }

    pub fn lib_dir(self) -> PathBuf {
        project_root().join(".magic").join("lib").join(self.name())
    }

    pub fn built_library_path(self) -> PathBuf {
        self.cache_dir()
            .join(BuildProfile::hot_reload().cargo_dir_name())
            .join(self.library_name())
    }

    pub fn library_path(self) -> PathBuf {
        self.lib_dir()
            .join(BuildProfile::hot_reload().name())
            .join(self.library_name())
    }

    pub fn install_library(self) -> Result<PathBuf, String> {
        let source = self.built_library_path();
        if !source.is_file() {
            return Err(format!("built library not found at {}", source.display()));
        }

        let destination = self.library_path();
        let destination_dir = destination
            .parent()
            .ok_or_else(|| format!("invalid library path: {}", destination.display()))?;

        fs::create_dir_all(destination_dir)
            .map_err(|error| format!("failed to create {}: {error}", destination_dir.display()))?;

        let temporary = destination_dir.join(format!(".{}.tmp", self.library_name()));
        fs::copy(&source, &temporary).map_err(|error| {
            format!(
                "failed to copy {} to {}: {error}",
                source.display(),
                temporary.display()
            )
        })?;

        #[cfg(windows)]
        if destination.exists() {
            fs::remove_file(&destination)
                .map_err(|error| format!("failed to replace {}: {error}", destination.display()))?;
        }

        fs::rename(&temporary, &destination).map_err(|error| {
            let _ = fs::remove_file(&temporary);
            format!(
                "failed to install {} to {}: {error}",
                source.display(),
                destination.display()
            )
        })?;

        Ok(destination)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Fast,
    Debug,
    Release,
}

impl BuildProfile {
    pub fn hot_reload() -> Self {
        if cfg!(debug_assertions) {
            if hot_reload_fast_enabled() {
                Self::Fast
            } else {
                Self::Debug
            }
        } else {
            Self::Release
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }

    pub fn cargo_dir_name(self) -> &'static str {
        match self {
            Self::Fast => "magic_fast",
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }

    pub fn cargo_args(self) -> &'static [&'static str] {
        match self {
            Self::Fast => &["--profile", "magic_fast"],
            Self::Debug => &[],
            Self::Release => &["--release"],
        }
    }
}

pub fn project_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn separate_targets_enabled() -> bool {
    std::env::var("MAGIC_SEPARATE_TARGETS")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn hot_reload_fast_enabled() -> bool {
    std::env::var("MAGIC_HOT_RELOAD_FAST")
        .map(|value| value != "0" && !value.eq_ignore_ascii_case("false"))
        .unwrap_or(true)
}

pub fn ensure_layout() -> std::io::Result<()> {
    let root = project_root().join(".magic");
    fs::create_dir_all(root)?;

    for target in BuildTarget::ALL {
        fs::create_dir_all(target.cache_dir())?;
        fs::create_dir_all(target.lib_dir())?;
        fs::create_dir_all(target.lib_dir().join(BuildProfile::hot_reload().name()))?;
    }

    Ok(())
}

pub fn editor_library_path() -> PathBuf {
    BuildTarget::Editor.library_path()
}
