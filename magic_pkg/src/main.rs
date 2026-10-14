use clap::{Parser, Subcommand};
use magic_core::{mgerror, mgprint, mgsuccess, mgwarn};
use magic_pkg::{PackageManager, PackageManifest, PackageTarget};
use std::path::PathBuf;

const TAG: &str = "PackageManager";

#[derive(Parser)]
#[command(name = "magic_pkg")]
#[command(about = "MagicBevy Package Manager CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, default_value = ".")]
    path: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new package
    Init {
        /// Category name (e.g. editor, runtime, games)
        #[arg(long, default_value = "editor")]
        category: String,
        /// Package identifier in vendor/name format (e.g. magicbevy/lighting_tools)
        package: String,
        /// Include editor target
        #[arg(long)]
        editor: bool,
        /// Include runtime target
        #[arg(long)]
        runtime: bool,
    },
    /// Install a package from a repository reference (vendor/name or vendor/name#version) or a local zip file
    Install {
        /// Repository reference or path to a local zip file
        source: String,
    },
    /// Remove an existing package
    Remove {
        /// Package identifier in vendor/name format
        package: String,
    },
    /// Enable a package
    Enable {
        /// Package identifier in vendor/name format
        package: String,
    },
    /// Disable a package
    Disable {
        /// Package identifier in vendor/name format
        package: String,
    },
    /// Check for available updates for installed packages
    Outdated,
    /// Update/upgrade one or all packages to the latest compatible version
    Update {
        /// Package identifier in vendor/name format (if omitted, updates all packages)
        package: Option<String>,
    },
    /// List all released versions and channels for a specific package
    Versions {
        /// Package identifier in vendor/name format (e.g. magicbevy/lighting_tools)
        package: String,
    },
    /// Update versions.toml of a local package with current version, magic version, and Unix timestamp
    Bump {
        /// Package identifier in vendor/name format (e.g. alirezapkg/test). If omitted, bumps all packages.
        package: Option<String>,
    },
    /// Sync package dependencies with Cargo manifests
    Sync,
    /// List all discovered packages
    List,
}

fn main() {
    let cli = Cli::parse();
    let manager = PackageManager::new(&cli.path);

    match cli.command {
        Commands::Init {
            category,
            package,
            editor,
            runtime,
        } => {
            let (vendor, name) = match parse_vendor_and_name(&package) {
                Ok(res) => res,
                Err(err) => {
                    mgerror!(TAG, "{}", err);
                    std::process::exit(1);
                }
            };

            let mut targets = Vec::new();
            if editor {
                targets.push(PackageTarget::Editor);
            }
            if runtime {
                targets.push(PackageTarget::Runtime);
            }
            if targets.is_empty() {
                targets.push(PackageTarget::Editor);
                targets.push(PackageTarget::Runtime);
            }

            match manager.init_package(&category, vendor, name, targets) {
                Ok((manifest, report)) => {
                    mgsuccess!(
                        TAG,
                        "Created package {} in category '{}' at {}",
                        manifest.id(),
                        manifest.category,
                        manifest.root.display()
                    );
                    mgprint!(
                        TAG,
                        "Synchronized {} enabled package(s) (Editor: {}, Runtime: {}, updated {} manifest files).",
                        report.enabled,
                        report.editor_packages,
                        report.runtime_packages,
                        report.updated_manifests
                    );
                }
                Err(err) => {
                    mgerror!(TAG, "Failed to initialize package: {}", err);
                    std::process::exit(1);
                }
            }
        }

        Commands::Install { source } => {
            let file_path = PathBuf::from(&source);

            if file_path.is_file() {
                match manager.install_from_zip(&file_path) {
                    Ok(manifest) => {
                        handle_successful_install(&manager, manifest, "zip archive");
                    }
                    Err(err) => {
                        mgerror!(TAG, "Failed to install package from zip: {}", err);
                        std::process::exit(1);
                    }
                }
            } else {
                let parts: Vec<&str> = source.split('#').collect();
                if parts.is_empty() || parts[0].is_empty() {
                    mgerror!(
                        TAG,
                        "Invalid install source. Expected 'vendor/name' or 'vendor/name#version' or a local zip file."
                    );
                    std::process::exit(1);
                }
                let package_part = parts[0];
                let version = if parts.len() > 1 && !parts[1].is_empty() {
                    parts[1]
                } else {
                    ""
                };

                let (vendor, name) = match parse_vendor_and_name(package_part) {
                    Ok(res) => res,
                    Err(err) => {
                        mgerror!(TAG, "{}", err);
                        std::process::exit(1);
                    }
                };

                match manager.install_repo(vendor, name, version) {
                    Ok(manifest) => {
                        handle_successful_install(&manager, manifest, "repository");
                    }
                    Err(err) => {
                        mgerror!(TAG, "Failed to install package: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::Remove { package } => {
            let (vendor, name) = match parse_vendor_and_name(&package) {
                Ok(res) => res,
                Err(err) => {
                    mgerror!(TAG, "{}", err);
                    std::process::exit(1);
                }
            };

            match manager.remove_package(vendor, name) {
                Ok((pkg_id, report)) => {
                    mgsuccess!(TAG, "Removed package {}.", pkg_id);
                    mgprint!(
                        TAG,
                        "Synchronized workspace (Remaining enabled: {}, updated {} manifest files).",
                        report.enabled,
                        report.updated_manifests
                    );
                }
                Err(err) => {
                    mgerror!(TAG, "Failed to remove package: {}", err);
                    std::process::exit(1);
                }
            }
        }

        Commands::Enable { package } => {
            set_status(&manager, &package, true);
        }

        Commands::Disable { package } => {
            set_status(&manager, &package, false);
        }

        Commands::Outdated => match manager.discover() {
            Ok(packages) => {
                if packages.is_empty() {
                    mgwarn!(TAG, "No packages found to check for updates.");
                } else {
                    mgprint!(TAG, "Checking for compatible package updates...");
                    let mut outdated_list = Vec::new();
                    for pkg in packages {
                        match manager.resolve_latest_compatible_version(&pkg.vendor, &pkg.name) {
                            Ok(latest_ver) => {
                                if latest_ver != pkg.version {
                                    outdated_list.push(magic_pkg::utils::OutdatedInfo {
                                        id: pkg.id(),
                                        current_version: pkg.version,
                                        latest_version: latest_ver,
                                    });
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    if outdated_list.is_empty() {
                        mgsuccess!(TAG, "All checked packages are up to date!");
                    } else {
                        mgprint!(
                            TAG,
                            "Found {} outdated package(s). Run 'magic_pkg update' to upgrade.",
                            outdated_list.len()
                        );
                        magic_pkg::utils::print_outdated_table(&outdated_list);
                    }
                }
            }
            Err(err) => {
                mgerror!(TAG, "Failed to check updates: {}", err);
                std::process::exit(1);
            }
        },

        Commands::Update { package } => {
            if let Some(pkg_str) = package {
                let (vendor, name) = match parse_vendor_and_name(&pkg_str) {
                    Ok(res) => res,
                    Err(err) => {
                        mgerror!(TAG, "{}", err);
                        std::process::exit(1);
                    }
                };
                match manager.install_repo(vendor, name, "") {
                    Ok(manifest) => {
                        handle_successful_install(&manager, manifest, "repository (updated)");
                    }
                    Err(err) => {
                        mgerror!(TAG, "Failed to update package {}: {}", pkg_str, err);
                        std::process::exit(1);
                    }
                }
            } else {
                match manager.discover() {
                    Ok(packages) => {
                        if packages.is_empty() {
                            mgwarn!(TAG, "No packages found to update.");
                        } else {
                            mgprint!(TAG, "Updating all discovered packages...");
                            for pkg in packages {
                                mgprint!(TAG, "Updating {}/{}...", pkg.vendor, pkg.name);
                                if let Err(e) = manager.install_repo(&pkg.vendor, &pkg.name, "") {
                                    mgwarn!(
                                        TAG,
                                        "Failed to update {}/{}: {}",
                                        pkg.vendor,
                                        pkg.name,
                                        e
                                    );
                                }
                            }
                            match manager.sync() {
                                Ok(_report) => {
                                    mgsuccess!(
                                        TAG,
                                        "All packages updated and workspace synchronized successfully."
                                    );
                                }
                                Err(e) => {
                                    mgerror!(TAG, "Packages updated, but sync failed: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        mgerror!(TAG, "Failed to list packages for update: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::Versions { package } => {
            let (vendor, name) = match parse_vendor_and_name(&package) {
                Ok(res) => res,
                Err(err) => {
                    mgerror!(TAG, "{}", err);
                    std::process::exit(1);
                }
            };

            let installed_version = manager.discover().ok().and_then(|packages| {
                packages
                    .into_iter()
                    .find(|p| p.vendor == vendor && p.name == name)
                    .map(|p| p.version)
            });

            mgprint!(TAG, "Fetching releases for {}/{}...", vendor, name);
            match manager.get_package_releases(vendor, name) {
                Ok(releases_file) => {
                    if releases_file.releases.is_empty() {
                        mgwarn!(TAG, "No releases found for package {}/{}", vendor, name);
                    } else {
                        magic_pkg::utils::print_releases_table(
                            &releases_file.package.id,
                            &releases_file.releases,
                            installed_version.as_deref(),
                        );
                    }
                }
                Err(err) => {
                    mgerror!(TAG, "Failed to fetch releases: {}", err);
                    std::process::exit(1);
                }
            }
        }

        Commands::Bump { package } => {
            if let Some(pkg_str) = package {
                let (vendor, name) = match parse_vendor_and_name(&pkg_str) {
                    Ok(res) => res,
                    Err(err) => {
                        mgerror!(TAG, "{}", err);
                        std::process::exit(1);
                    }
                };
                match manager.bump_package(vendor, name) {
                    Ok(()) => {
                        mgsuccess!(
                            TAG,
                            "Successfully updated versions.toml for package {}/{} with current timestamp and magic version.",
                            vendor,
                            name
                        );
                    }
                    Err(err) => {
                        mgerror!(TAG, "Failed to update versions.toml: {}", err);
                        std::process::exit(1);
                    }
                }
            } else {
                match manager.bump_all() {
                    Ok(count) => {
                        mgsuccess!(
                            TAG,
                            "Successfully updated versions.toml for all {} discovered package(s).",
                            count
                        );
                    }
                    Err(err) => {
                        mgerror!(TAG, "Failed to update versions.toml files: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::Sync => match manager.sync() {
            Ok(report) => {
                mgsuccess!(
                    TAG,
                    "Sync complete: {} discovered, {} enabled (Editor: {}, Runtime: {}, updated {} manifest files).",
                    report.discovered,
                    report.enabled,
                    report.editor_packages,
                    report.runtime_packages,
                    report.updated_manifests
                );
            }
            Err(err) => {
                mgerror!(TAG, "Sync failed: {}", err);
                std::process::exit(1);
            }
        },

        Commands::List => match manager.discover() {
            Ok(packages) => {
                if packages.is_empty() {
                    mgwarn!(TAG, "No packages found in packages directory.");
                } else {
                    mgprint!(TAG, "Discovered {} package(s):", packages.len());
                    magic_pkg::utils::print_packages_table(&packages);
                }
            }
            Err(err) => {
                mgerror!(TAG, "Failed to list packages: {}", err);
                std::process::exit(1);
            }
        },
    }
}

fn handle_successful_install(
    manager: &PackageManager,
    manifest: PackageManifest,
    source_type: &str,
) {
    match manager.sync() {
        Ok(report) => {
            mgsuccess!(
                TAG,
                "Successfully installed package {} version {} from {}.",
                manifest.id(),
                manifest.version,
                source_type
            );
            mgprint!(
                TAG,
                "Synchronized workspace (Enabled: {}, Editor: {}, Runtime: {}, updated {} manifest & code files).",
                report.enabled,
                report.editor_packages,
                report.runtime_packages,
                report.updated_manifests + report.updated_code_files
            );
        }
        Err(err) => {
            mgerror!(TAG, "Package installed, but workspace sync failed: {}", err);
            std::process::exit(1);
        }
    }
}

fn set_status(manager: &PackageManager, package: &str, enabled: bool) {
    let (vendor, name) = match parse_vendor_and_name(package) {
        Ok(res) => res,
        Err(err) => {
            mgerror!(TAG, "{}", err);
            std::process::exit(1);
        }
    };

    match manager.set_package_status(vendor, name, enabled) {
        Ok((pkg_id, report)) => {
            let action = if enabled { "Enabled" } else { "Disabled" };
            mgsuccess!(TAG, "{} package {}.", action, pkg_id);
            mgprint!(
                TAG,
                "Synchronized workspace (Enabled: {}, updated {} manifest files).",
                report.enabled,
                report.updated_manifests
            );
        }
        Err(err) => {
            mgerror!(TAG, "Failed to update package status: {}", err);
            std::process::exit(1);
        }
    }
}

fn parse_vendor_and_name(input: &str) -> Result<(&str, &str), String> {
    let parts: Vec<&str> = input.split('/').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Ok((parts[0], parts[1]))
    } else {
        Err(format!(
            "Invalid package format '{input}'. Expected 'vendor/name' (e.g. 'magicbevy/lighting_tools')"
        ))
    }
}
