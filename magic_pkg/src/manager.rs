use crate::error::PackageError;
use crate::manifest::{
    PackageManifest, PackageReleasesFile, PackageTarget, ProjectPackagesManifest, RawManifest,
    ReleaseEntry, ReleasePackageMeta,
};
use crate::utils::{
    copy_dir_all, extract_zip_archive, file_name, find_crate_manifest, find_workspace_root,
    package_name, read_directories, update_editor_lib_register, update_manifest_dependencies,
    update_runtime_lib_register, uuid_lite, validate_segment,
};
use magic_core::mgwarn;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const TAG: &str = "PackageManager";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub discovered: usize,
    pub enabled: usize,
    pub editor_packages: usize,
    pub runtime_packages: usize,
    pub updated_manifests: usize,
    pub updated_code_files: usize,
    pub installed_from_cache: usize,
}

pub struct PackageManager {
    packages_root: PathBuf,
    packages_manifest: PathBuf,
    editor_manifest: PathBuf,
    runtime_manifest: PathBuf,
    editor_lib_rs: PathBuf,
    runtime_lib_rs: PathBuf,
}

impl PackageManager {
    pub fn new(start_path: impl Into<PathBuf>) -> Self {
        let workspace_root = find_workspace_root(&start_path.into());
        let packages_root = workspace_root.join("packages");
        let packages_manifest = packages_root.join("packages.toml");

        let editor_manifest = find_crate_manifest(&workspace_root, "magic_pkg_editor")
            .unwrap_or_else(|| workspace_root.join("magic_editor/magic_pkg_editor/Cargo.toml"));
        let runtime_manifest = find_crate_manifest(&workspace_root, "magic_pkg_runtime")
            .unwrap_or_else(|| workspace_root.join("magic_runtime/magic_pkg_runtime/Cargo.toml"));

        let editor_lib_rs = editor_manifest
            .parent()
            .map(|p| p.join("src/lib.rs"))
            .unwrap_or_else(|| workspace_root.join("magic_editor/magic_pkg_editor/src/lib.rs"));

        let runtime_lib_rs = runtime_manifest
            .parent()
            .map(|p| p.join("src/lib.rs"))
            .unwrap_or_else(|| workspace_root.join("magic_runtime/magic_pkg_runtime/src/lib.rs"));

        Self {
            packages_root,
            packages_manifest,
            editor_manifest,
            runtime_manifest,
            editor_lib_rs,
            runtime_lib_rs,
        }
    }

    pub fn global_magic_dir() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        home.join(".magic-bevy")
    }

    pub fn global_cache_dir() -> PathBuf {
        Self::global_magic_dir().join("packages")
    }

    pub fn packages_manifest_path(&self) -> &Path {
        &self.packages_manifest
    }

    pub fn load_project_manifest(&self) -> Result<ProjectPackagesManifest, PackageError> {
        ProjectPackagesManifest::load_from_file(&self.packages_manifest)
    }

    pub fn save_project_manifest(
        &self,
        manifest: &ProjectPackagesManifest,
    ) -> Result<(), PackageError> {
        manifest.save_to_file(&self.packages_manifest)
    }

    pub fn update_versions_file(package_root: &Path) -> Result<(), PackageError> {
        let package_toml_path = package_root.join("package.toml");
        let content = fs::read_to_string(&package_toml_path).map_err(|s| PackageError::Io {
            path: package_toml_path.clone(),
            source: s,
        })?;
        let raw: RawManifest = toml::from_str(&content).map_err(|s| PackageError::Manifest {
            path: package_toml_path.clone(),
            source: s,
        })?;

        let versions_path = package_root.join("versions.toml");
        let mut versions_file = if versions_path.is_file() {
            let c = fs::read_to_string(&versions_path).map_err(|s| PackageError::Io {
                path: versions_path.clone(),
                source: s,
            })?;
            toml::from_str::<PackageReleasesFile>(&c).unwrap_or(PackageReleasesFile {
                package: ReleasePackageMeta {
                    id: format!("{}/{}", raw.vendor.as_deref().unwrap_or("custom"), raw.name),
                },
                releases: Vec::new(),
            })
        } else {
            PackageReleasesFile {
                package: ReleasePackageMeta {
                    id: format!("{}/{}", raw.vendor.as_deref().unwrap_or("custom"), raw.name),
                },
                releases: Vec::new(),
            }
        };

        let now_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let channel_str = raw.channel.clone();
        let magic_ver = raw.magic.unwrap_or_else(|| "0.1.0".to_string());

        if let Some(existing) = versions_file
            .releases
            .iter_mut()
            .find(|r| r.version == raw.version)
        {
            existing.magic = magic_ver;
            existing.timestamp = now_timestamp;
            existing.channel = channel_str;
        } else {
            versions_file.releases.push(ReleaseEntry {
                version: raw.version.clone(),
                magic: magic_ver,
                timestamp: now_timestamp,
                channel: channel_str,
            });
        }

        versions_file
            .releases
            .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let serialized =
            toml::to_string_pretty(&versions_file).map_err(|s| PackageError::InvalidManifest {
                path: versions_path.clone(),
                message: format!("Failed to serialize versions.toml: {s}"),
            })?;

        fs::write(&versions_path, serialized).map_err(|s| PackageError::Io {
            path: versions_path.clone(),
            source: s,
        })?;

        Ok(())
    }

    pub fn bump_package(&self, vendor: &str, name: &str) -> Result<(), PackageError> {
        let packages = self.discover()?;
        let pkg = packages
            .iter()
            .find(|p| p.vendor == vendor && p.name == name)
            .ok_or_else(|| {
                PackageError::InvalidArgument(format!("Package '{vendor}/{name}' not found"))
            })?;
        Self::update_versions_file(&pkg.root)
    }

    pub fn bump_all(&self) -> Result<usize, PackageError> {
        let packages = self.discover()?;
        let mut count = 0;
        for pkg in packages {
            Self::update_versions_file(&pkg.root)?;
            count += 1;
        }
        Ok(count)
    }

    pub fn resolve_latest_compatible_version(
        &self,
        vendor: &str,
        name: &str,
    ) -> Result<String, PackageError> {
        let workspace_root = find_workspace_root(&self.packages_root);
        let engine_version = crate::utils::get_workspace_engine_version(&workspace_root)
            .unwrap_or_else(|| "0.1.0".to_string());

        let url_main = format!(
            "https://raw.githubusercontent.com/{}/{}/main/versions.toml",
            vendor, name
        );
        let url_master = format!(
            "https://raw.githubusercontent.com/{}/{}/master/versions.toml",
            vendor, name
        );

        let content_res = ureq::get(&url_main)
            .call()
            .and_then(|res| res.into_body().read_to_string());
        let content = match content_res {
            Ok(c) => c,
            Err(_) => ureq::get(&url_master)
                .call()
                .map_err(|e| {
                    PackageError::DownloadError(format!(
                        "Failed to fetch versions.toml for {}/{}: {}",
                        vendor, name, e
                    ))
                })?
                .into_body()
                .read_to_string()
                .map_err(|e| PackageError::DownloadError(e.to_string()))?,
        };

        let releases_file: PackageReleasesFile =
            toml::from_str(&content).map_err(|s| PackageError::InvalidManifest {
                path: PathBuf::from("versions.toml"),
                message: format!("Failed to parse remote versions.toml: {s}"),
            })?;

        let mut releases = releases_file.releases;

        releases.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        for release in &releases {
            if release.magic == engine_version && release.channel == "stable" {
                return Ok(release.version.clone());
            }
        }

        for release in &releases {
            if release.magic == engine_version {
                return Ok(release.version.clone());
            }
        }

        Err(PackageError::InvalidManifest {
            path: self.packages_manifest.clone(),
            message: format!(
                "No compatible version found for package {}/{} with engine version '{}'",
                vendor, name, engine_version
            ),
        })
    }

    pub fn get_package_releases(
        &self,
        vendor: &str,
        name: &str,
    ) -> Result<PackageReleasesFile, PackageError> {
        let url_main = format!(
            "https://raw.githubusercontent.com/{}/{}/main/versions.toml",
            vendor, name
        );
        let url_master = format!(
            "https://raw.githubusercontent.com/{}/{}/master/versions.toml",
            vendor, name
        );

        let content_res = ureq::get(&url_main)
            .call()
            .and_then(|res| res.into_body().read_to_string());
        let content = match content_res {
            Ok(c) => c,
            Err(_) => ureq::get(&url_master)
                .call()
                .map_err(|e| {
                    PackageError::DownloadError(format!(
                        "Failed to fetch versions.toml for {}/{}: {}",
                        vendor, name, e
                    ))
                })?
                .into_body()
                .read_to_string()
                .map_err(|e| PackageError::DownloadError(e.to_string()))?,
        };

        let releases_file: PackageReleasesFile =
            toml::from_str(&content).map_err(|s| PackageError::InvalidManifest {
                path: PathBuf::from("versions.toml"),
                message: format!("Failed to parse remote versions.toml: {s}"),
            })?;

        Ok(releases_file)
    }

    pub fn install_from_zip(&self, zip_path: &Path) -> Result<PackageManifest, PackageError> {
        let temp_dir = std::env::temp_dir().join(format!("magic_install_{}", uuid_lite()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).map_err(|s| PackageError::Io {
            path: temp_dir.clone(),
            source: s,
        })?;

        extract_zip_archive(zip_path, &temp_dir)?;

        let manifest_dir = if temp_dir.join("package.toml").is_file() {
            temp_dir.clone()
        } else {
            let mut found = None;
            if let Ok(entries) = fs::read_dir(&temp_dir) {
                for entry in entries.filter_map(Result::ok) {
                    if entry.path().is_dir() && entry.path().join("package.toml").is_file() {
                        found = Some(entry.path());
                        break;
                    }
                }
            }
            found.ok_or_else(|| PackageError::InvalidManifest {
                path: temp_dir.clone(),
                message: "No package.toml found in zip archive".into(),
            })?
        };

        let manifest_content =
            fs::read_to_string(manifest_dir.join("package.toml")).map_err(|s| {
                PackageError::Io {
                    path: manifest_dir.join("package.toml"),
                    source: s,
                }
            })?;
        let raw: RawManifest =
            toml::from_str(&manifest_content).map_err(|s| PackageError::Manifest {
                path: manifest_dir.join("package.toml"),
                source: s,
            })?;

        let category = raw.category.unwrap_or_else(|| "editor".to_string());
        let vendor = raw.vendor.as_deref().unwrap_or("custom");

        let dest_package_dir = self
            .packages_root
            .join(&category)
            .join(vendor)
            .join(&raw.name);

        if dest_package_dir.exists() {
            let _ = fs::remove_dir_all(&dest_package_dir);
        }
        fs::create_dir_all(dest_package_dir.parent().unwrap()).map_err(|s| PackageError::Io {
            path: dest_package_dir.clone(),
            source: s,
        })?;

        copy_dir_all(&manifest_dir, &dest_package_dir)?;

        let cache_dir = Self::global_cache_dir().join(vendor).join(&raw.name);
        let _ = fs::create_dir_all(&cache_dir);
        let cache_zip_target = cache_dir.join(format!("{}.zip", raw.version));

        if zip_path != cache_zip_target {
            let _ = fs::copy(zip_path, &cache_zip_target);
        }

        let _ = fs::remove_dir_all(&temp_dir);

        let mut proj_manifest = self.load_project_manifest()?;
        proj_manifest
            .dependencies
            .insert(format!("{vendor}/{}", raw.name), raw.version.clone());

        for (dep_id, dep_ver) in &raw.dependencies {
            if !proj_manifest.dependencies.contains_key(dep_id) {
                proj_manifest
                    .dependencies
                    .insert(dep_id.clone(), dep_ver.clone());
            }
        }
        self.save_project_manifest(&proj_manifest)?;

        self.read_manifest(&category, vendor, dest_package_dir)
    }

    pub fn install_from_url(&self, url: &str) -> Result<PackageManifest, PackageError> {
        magic_core::mgprint!(TAG, "Downloading package from {} ...", url);
        let response = ureq::get(url)
            .call()
            .map_err(|e| PackageError::DownloadError(e.to_string()))?;

        let total_size = response
            .headers()
            .get("Content-Length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        let temp_zip_path = std::env::temp_dir().join(format!("download_{}.zip", uuid_lite()));
        let mut out_file = fs::File::create(&temp_zip_path).map_err(|s| PackageError::Io {
            path: temp_zip_path.clone(),
            source: s,
        })?;

        let reader = response.into_body().into_reader();
        let mut buffered_reader = io::BufReader::new(reader);
        let mut downloaded: u64 = 0;
        let mut buffer = [0u8; 8192];

        loop {
            let n = std::io::Read::read(&mut buffered_reader, &mut buffer).map_err(|s| {
                PackageError::Io {
                    path: temp_zip_path.clone(),
                    source: s,
                }
            })?;
            if n == 0 {
                break;
            }
            std::io::Write::write_all(&mut out_file, &buffer[..n]).map_err(|s| {
                PackageError::Io {
                    path: temp_zip_path.clone(),
                    source: s,
                }
            })?;
            downloaded += n as u64;

            if let Some(total) = total_size {
                let percent = (downloaded as f64 / total as f64) * 100.0;
                print!(
                    "\r    [Downloading] {} / {} bytes ({:.1}%)",
                    downloaded, total, percent
                );
                use std::io::Write;
                let _ = std::io::stdout().flush();
            } else {
                print!("\r    [Downloading] {} bytes downloaded...", downloaded);
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
        }
        println!();

        let result = self.install_from_zip(&temp_zip_path);
        let _ = fs::remove_file(temp_zip_path);
        result
    }

    pub fn install_repo(
        &self,
        vendor: &str,
        name: &str,
        version_tag: &str,
    ) -> Result<PackageManifest, PackageError> {
        let resolved_version = if version_tag.is_empty() || version_tag == "latest" {
            magic_core::mgprint!(
                TAG,
                "Resolving latest compatible version for {}/{} ...",
                vendor,
                name
            );
            self.resolve_latest_compatible_version(vendor, name)?
        } else {
            version_tag.to_string()
        };

        let clean_version = resolved_version
            .strip_prefix('v')
            .unwrap_or(&resolved_version);
        let cached_zip = Self::global_cache_dir()
            .join(vendor)
            .join(name)
            .join(format!("{clean_version}.zip"));

        if cached_zip.is_file() {
            magic_core::mgprint!(
                TAG,
                "Installing package {}/{} v{} from local cache ({}) ...",
                vendor,
                name,
                clean_version,
                cached_zip.display()
            );
            return self.install_from_zip(&cached_zip);
        }

        let tag = if resolved_version.starts_with('v') {
            resolved_version
        } else {
            format!("v{}", resolved_version)
        };

        let url = format!(
            "https://github.com/{}/{}/archive/refs/tags/{}.zip",
            vendor, name, tag
        );

        self.install_from_url(&url)
    }

    pub fn discover(&self) -> Result<Vec<PackageManifest>, PackageError> {
        if !self.packages_root.exists() {
            return Ok(Vec::new());
        }

        let mut packages = Vec::new();
        let mut identifiers = BTreeSet::new();

        for category_entry in read_directories(&self.packages_root)? {
            let category = file_name(&category_entry)?;
            for vendor_entry in read_directories(&category_entry)? {
                let vendor = file_name(&vendor_entry)?;
                for package_root in read_directories(&vendor_entry)? {
                    let package = self.read_manifest(&category, &vendor, package_root)?;
                    if !identifiers.insert(package.dependency_identifier()) {
                        return Err(PackageError::DuplicateId(package.id()));
                    }
                    packages.push(package);
                }
            }
        }
        packages.sort_by(|left, right| left.id().cmp(&right.id()));
        Ok(packages)
    }

    pub fn sync(&self) -> Result<SyncReport, PackageError> {
        let mut report = SyncReport::default();

        loop {
            let mut proj_manifest = self.load_project_manifest()?;
            let mut missing_or_new = false;

            for (pkg_id, version_or_url) in proj_manifest.dependencies.clone() {
                let parts: Vec<&str> = pkg_id.split('/').collect();
                if parts.len() != 2 {
                    continue;
                }
                let vendor = parts[0];
                let name = parts[1];
                let mut exists = false;
                if let Ok(categories) = read_directories(&self.packages_root) {
                    for cat_dir in categories {
                        if cat_dir.join(vendor).join(name).exists() {
                            exists = true;
                            break;
                        }
                    }
                }

                if !exists {
                    missing_or_new = true;
                    if version_or_url.starts_with("http://")
                        || version_or_url.starts_with("https://")
                    {
                        self.install_from_url(&version_or_url)?;
                    } else {
                        self.install_repo(vendor, name, &version_or_url)?;
                    }
                }
            }

            let packages = self.discover()?;
            let mut proj_manifest_updated = false;

            for package in &packages {
                let key = format!("{}/{}", package.vendor, package.name);
                if !proj_manifest.dependencies.contains_key(&key) {
                    proj_manifest
                        .dependencies
                        .insert(key, package.version.clone());
                    proj_manifest_updated = true;
                }

                let manifest_path = package.root.join("package.toml");
                if let Ok(content) = fs::read_to_string(&manifest_path) {
                    if let Ok(raw) = toml::from_str::<RawManifest>(&content) {
                        for (dep_id, dep_ver) in raw.dependencies {
                            if !proj_manifest.dependencies.contains_key(&dep_id) {
                                proj_manifest.dependencies.insert(dep_id, dep_ver);
                                proj_manifest_updated = true;
                                missing_or_new = true;
                            }
                        }
                    }
                }
            }

            if proj_manifest_updated {
                self.save_project_manifest(&proj_manifest)?;
                report.updated_manifests += 1;
            }

            if !missing_or_new {
                break;
            }
        }

        let packages = self.discover()?;
        report.discovered = packages.len();

        let mut editor_deps = Vec::new();
        let mut runtime_deps = Vec::new();

        let mut editor_aliases = Vec::new();
        let mut runtime_aliases = Vec::new();

        for package in packages.into_iter().filter(|package| package.enabled) {
            report.enabled += 1;
            let alias = format!("package_{}", package.dependency_identifier());

            for target in &package.targets {
                let crate_path = package.root.join(target.directory());
                let crate_manifest = crate_path.join("Cargo.toml");
                let crate_name = package_name(&crate_manifest)?;

                match target {
                    PackageTarget::Editor => {
                        report.editor_packages += 1;
                        editor_deps.push((alias.clone(), crate_name, crate_path));
                        editor_aliases.push(alias.clone());
                    }
                    PackageTarget::Runtime => {
                        report.runtime_packages += 1;
                        runtime_deps.push((alias.clone(), crate_name, crate_path));
                        runtime_aliases.push(alias.clone());
                    }
                }
            }
        }

        if self.editor_manifest.exists() {
            if update_manifest_dependencies(&self.editor_manifest, &editor_deps)? {
                report.updated_manifests += 1;
            }
        } else {
            mgwarn!(
                "magic_pkg",
                "Editor manifest not found at {}",
                self.editor_manifest.display()
            );
        }

        if self.runtime_manifest.exists() {
            if update_manifest_dependencies(&self.runtime_manifest, &runtime_deps)? {
                report.updated_manifests += 1;
            }
        } else {
            mgwarn!(
                "magic_pkg",
                "Runtime manifest not found at {}",
                self.runtime_manifest.display()
            );
        }

        if update_editor_lib_register(&self.editor_lib_rs, &editor_aliases)? {
            report.updated_code_files += 1;
        }

        if update_runtime_lib_register(&self.runtime_lib_rs, &runtime_aliases)? {
            report.updated_code_files += 1;
        }

        Ok(report)
    }

    pub fn init_package(
        &self,
        category: &str,
        vendor: &str,
        name: &str,
        targets: impl IntoIterator<Item = PackageTarget>,
    ) -> Result<(PackageManifest, SyncReport), PackageError> {
        validate_segment("category", category)?;
        validate_segment("vendor", vendor)?;
        validate_segment("package name", name)?;
        let targets = targets.into_iter().collect::<BTreeSet<_>>();
        if targets.is_empty() {
            return Err(PackageError::InvalidArgument(
                "at least one target is required".into(),
            ));
        }

        let root = self.packages_root.join(category).join(vendor).join(name);
        if root.exists() {
            return Err(PackageError::AlreadyExists(root));
        }

        let previews_dir = root.join("assets/previews_pkg");
        fs::create_dir_all(&previews_dir).map_err(|source| PackageError::Io {
            path: previews_dir.clone(),
            source,
        })?;
        fs::write(previews_dir.join(".gitkeep"), "").map_err(|source| PackageError::Io {
            path: previews_dir.join(".gitkeep"),
            source,
        })?;

        let workspace_root = find_workspace_root(&self.packages_root);
        let magic_version = crate::utils::get_workspace_engine_version(&workspace_root)
            .unwrap_or_else(|| "0.1.0".to_string());

        let target_values = targets
            .iter()
            .map(|target| format!("\"{}\"", target.manifest_value()))
            .collect::<Vec<_>>()
            .join(", ");

        let manifest = format!(
            "name = \"{name}\"\nvendor = \"{vendor}\"\nversion = \"0.1.0\"\ncategory = \"{category}\"\nmagic = \"{magic_version}\"\nchannel = \"stable\"\ndescription = \"Write your package description here\"\nkeywords = []\nstatus = \"enabled\"\ntargets = [{target_values}]\n\n[dependencies]\n"
        );
        fs::write(root.join("package.toml"), manifest).map_err(|source| PackageError::Io {
            path: root.join("package.toml"),
            source,
        })?;

        for target in &targets {
            self.write_target_template(&root, vendor, name, *target)?;
        }

        let _ = Self::update_versions_file(&root);

        let mut proj_manifest = self.load_project_manifest()?;
        proj_manifest
            .dependencies
            .insert(format!("{vendor}/{name}"), "0.1.0".to_string());
        self.save_project_manifest(&proj_manifest)?;

        let manifest_data = self.read_manifest(category, vendor, root)?;
        let report = self.sync()?;
        Ok((manifest_data, report))
    }

    pub fn remove_package(
        &self,
        vendor: &str,
        name: &str,
    ) -> Result<(String, SyncReport), PackageError> {
        validate_segment("vendor", vendor)?;
        validate_segment("package name", name)?;

        let mut target_root = None;
        let mut target_category_dir = None;

        if let Ok(categories) = read_directories(&self.packages_root) {
            for cat_dir in categories {
                let pkg_path = cat_dir.join(vendor).join(name);
                if pkg_path.exists() {
                    target_root = Some(pkg_path);
                    target_category_dir = Some(cat_dir);
                    break;
                }
            }
        }

        let root = target_root.ok_or_else(|| {
            PackageError::InvalidArgument(format!("Package '{vendor}/{name}' does not exist"))
        })?;

        fs::remove_dir_all(&root).map_err(|source| PackageError::Io {
            path: root.clone(),
            source,
        })?;

        let vendor_dir = root.parent().unwrap();
        if let Ok(mut entries) = fs::read_dir(vendor_dir) {
            if entries.next().is_none() {
                let _ = fs::remove_dir(vendor_dir);

                if let Some(cat_dir) = target_category_dir {
                    if let Ok(mut cat_entries) = fs::read_dir(&cat_dir) {
                        if cat_entries.next().is_none() {
                            let _ = fs::remove_dir(&cat_dir);
                        }
                    }
                }
            }
        }

        if let Ok(mut proj_manifest) = self.load_project_manifest() {
            proj_manifest
                .dependencies
                .remove(&format!("{vendor}/{name}"));
            let _ = self.save_project_manifest(&proj_manifest);
        }

        let report = self.sync()?;

        Ok((format!("{vendor}/{name}"), report))
    }

    pub fn set_package_status(
        &self,
        vendor: &str,
        name: &str,
        enabled: bool,
    ) -> Result<(String, SyncReport), PackageError> {
        validate_segment("vendor", vendor)?;
        validate_segment("package name", name)?;

        let mut target_manifest_path = None;
        if let Ok(categories) = read_directories(&self.packages_root) {
            for cat_dir in categories {
                let manifest_p = cat_dir.join(vendor).join(name).join("package.toml");
                if manifest_p.is_file() {
                    target_manifest_path = Some(manifest_p);
                    break;
                }
            }
        }

        let manifest_path = target_manifest_path.ok_or_else(|| {
            PackageError::InvalidArgument(format!("Package '{vendor}/{name}' does not exist"))
        })?;

        let content = fs::read_to_string(&manifest_path).map_err(|source| PackageError::Io {
            path: manifest_path.clone(),
            source,
        })?;

        let mut toml_val: toml::Value =
            toml::from_str(&content).map_err(|source| PackageError::Manifest {
                path: manifest_path.clone(),
                source,
            })?;

        if let Some(table) = toml_val.as_table_mut() {
            let status_str = if enabled { "enabled" } else { "disabled" };
            table.insert(
                "status".to_string(),
                toml::Value::String(status_str.to_string()),
            );
            table.remove("enabled");
        }

        let updated_toml =
            toml::to_string_pretty(&toml_val).map_err(|source| PackageError::InvalidManifest {
                path: manifest_path.clone(),
                message: format!("Failed to serialize TOML: {source}"),
            })?;

        fs::write(&manifest_path, updated_toml).map_err(|source| PackageError::Io {
            path: manifest_path,
            source,
        })?;

        let report = self.sync()?;

        Ok((format!("{vendor}/{name}"), report))
    }

    fn write_target_template(
        &self,
        package_root: &Path,
        vendor: &str,
        name: &str,
        target: PackageTarget,
    ) -> Result<(), PackageError> {
        let target_root = package_root.join(target.directory());
        let source_root = target_root.join("src");
        fs::create_dir_all(&source_root).map_err(|source| PackageError::Io {
            path: source_root.clone(),
            source,
        })?;
        let crate_name = format!(
            "magic_package_{}_{}_{}",
            crate::utils::cargo_identifier(vendor),
            crate::utils::cargo_identifier(name),
            target.manifest_value()
        );

        let workspace_root = find_workspace_root(&self.packages_root);
        let core_path = workspace_root.join("magic_core");
        let rel_core_path = crate::utils::relative_path(&target_root, &core_path);
        let core_path_str = crate::utils::toml_path(&rel_core_path);

        let cargo_toml = format!(
            "[package]\nname = \"{crate_name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\npublish = false\n\n[dependencies]\nmagic_core = {{ path = \"{core_path_str}\" }}\n",
        );
        fs::write(target_root.join("Cargo.toml"), cargo_toml).map_err(|source| {
            PackageError::Io {
                path: target_root.join("Cargo.toml"),
                source,
            }
        })?;

        let source = match target {
            PackageTarget::Editor => {
                "use magic_core::api::EditorRegistry;\n\npub fn register(_registry: &mut EditorRegistry) {}\n"
            }
            PackageTarget::Runtime => {
                "use magic_core::api::RuntimeRegistry;\n\npub fn register(_registry: &mut RuntimeRegistry) {}\n"
            }
        };

        fs::write(source_root.join("lib.rs"), source).map_err(|source| PackageError::Io {
            path: source_root.join("lib.rs"),
            source,
        })?;
        Ok(())
    }

    fn read_manifest(
        &self,
        category: &str,
        vendor: &str,
        root: PathBuf,
    ) -> Result<PackageManifest, PackageError> {
        let path = root.join("package.toml");
        if !path.is_file() {
            return Err(PackageError::InvalidManifest {
                path,
                message: "package.toml is required".into(),
            });
        }
        let content = fs::read_to_string(&path).map_err(|source| PackageError::Io {
            path: path.clone(),
            source,
        })?;
        let raw: RawManifest =
            toml::from_str(&content).map_err(|source| PackageError::Manifest {
                path: path.clone(),
                source,
            })?;
        if raw.name.trim().is_empty() || raw.version.trim().is_empty() {
            return Err(PackageError::InvalidManifest {
                path,
                message: "name and version must not be empty".to_string(),
            });
        }

        if let Some(ref required_magic) = raw.magic {
            let workspace_root = find_workspace_root(&root);
            if let Some(engine_version) =
                crate::utils::get_workspace_engine_version(&workspace_root)
            {
                if required_magic != &engine_version {
                    return Err(PackageError::InvalidManifest {
                        path: path.clone(),
                        message: format!(
                            "Package requires magic engine version '{}', but workspace engine version is '{}'",
                            required_magic, engine_version
                        ),
                    });
                }
            }
        }

        let enabled = match raw.status.as_deref() {
            None => raw.enabled.unwrap_or(true),
            Some("enabled") if raw.enabled != Some(false) => true,
            Some("disabled") if raw.enabled != Some(true) => false,
            Some("enabled") | Some("disabled") => {
                return Err(PackageError::InvalidManifest {
                    path,
                    message: "status and enabled disagree".into(),
                });
            }
            Some(status) => {
                return Err(PackageError::InvalidManifest {
                    path,
                    message: format!("unsupported status '{status}'"),
                });
            }
        };
        let mut targets = BTreeSet::new();
        for target in raw.targets {
            match target.as_str() {
                "editor" => {
                    targets.insert(PackageTarget::Editor);
                }
                "runtime" => {
                    targets.insert(PackageTarget::Runtime);
                }
                _ => {
                    return Err(PackageError::InvalidManifest {
                        path,
                        message: format!("unsupported target '{target}'"),
                    });
                }
            }
        }
        if enabled && targets.is_empty() {
            return Err(PackageError::InvalidManifest {
                path,
                message: "enabled packages require at least one target".into(),
            });
        }
        for target in &targets {
            let crate_manifest = root.join(target.directory()).join("Cargo.toml");
            if !crate_manifest.is_file() {
                return Err(PackageError::InvalidManifest {
                    path: crate_manifest,
                    message: format!("{} target requires a Cargo crate", target.directory()),
                });
            }
        }
        Ok(PackageManifest {
            category: category.to_owned(),
            vendor: vendor.to_owned(),
            name: raw.name,
            version: raw.version,
            enabled,
            targets,
            root,
        })
    }
}
