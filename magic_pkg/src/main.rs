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
    /// Install a package from a repository reference (vendor/name#version) or a local zip file
    Install {
        /// Repository reference (e.g. magicbevy/lighting_tools#v0.1.0) or path to a local zip file
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
                        "Invalid install source. Expected 'vendor/name#version' or a valid local zip file path."
                    );
                    std::process::exit(1);
                }
                let package_part = parts[0];
                let version = if parts.len() > 1 && !parts[1].is_empty() {
                    parts[1]
                } else {
                    "v0.1.0"
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
                    print_packages_table(&packages);
                }
            }
            Err(err) => {
                mgerror!(TAG, "Failed to list packages: {}", err);
                std::process::exit(1);
            }
        },
    }
}

fn handle_successful_install(manager: &PackageManager, manifest: PackageManifest, source_type: &str) {
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

fn print_packages_table(packages: &[PackageManifest]) {
    let mut cat_w = 8;
    let mut id_w = 12;
    let mut ver_w = 7;
    let mut stat_w = 8;
    let mut targ_w = 10;

    for p in packages {
        cat_w = cat_w.max(p.category.len());
        id_w = id_w.max(p.id().len());
        ver_w = ver_w.max(p.version.len());
        let status_str = if p.enabled { "Enabled" } else { "Disabled" };
        stat_w = stat_w.max(status_str.len());

        let targets_str = p
            .targets
            .iter()
            .map(|t| match t {
                PackageTarget::Editor => "Editor",
                PackageTarget::Runtime => "Runtime",
            })
            .collect::<Vec<_>>()
            .join(", ");
        targ_w = targ_w.max(targets_str.len());
    }

    cat_w += 2;
    id_w += 2;
    ver_w += 2;
    stat_w += 2;
    targ_w += 2;

    let top = format!(
        "┌{}┬{}┬{}┬{}┬{}┐",
        "─".repeat(cat_w),
        "─".repeat(id_w),
        "─".repeat(ver_w),
        "─".repeat(stat_w),
        "─".repeat(targ_w)
    );
    let header = format!(
        "│ {:<cat_len$} │ {:<id_len$} │ {:<ver_len$} │ {:<stat_len$} │ {:<targ_len$} │",
        "Category",
        "Package ID",
        "Version",
        "Status",
        "Targets",
        cat_len = cat_w - 2,
        id_len = id_w - 2,
        ver_len = ver_w - 2,
        stat_len = stat_w - 2,
        targ_len = targ_w - 2
    );
    let sep = format!(
        "├{}┼{}┼{}┼{}┼{}┤",
        "─".repeat(cat_w),
        "─".repeat(id_w),
        "─".repeat(ver_w),
        "─".repeat(stat_w),
        "─".repeat(targ_w)
    );
    let bot = format!(
        "└{}┴{}┴{}┴{}┴{}┘",
        "─".repeat(cat_w),
        "─".repeat(id_w),
        "─".repeat(ver_w),
        "─".repeat(stat_w),
        "─".repeat(targ_w)
    );

    println!("    {top}");
    println!("    {header}");
    println!("    {sep}");
    for p in packages {
        let status_str = if p.enabled { "Enabled" } else { "Disabled" };
        let targets_str = p
            .targets
            .iter()
            .map(|t| match t {
                PackageTarget::Editor => "Editor",
                PackageTarget::Runtime => "Runtime",
            })
            .collect::<Vec<_>>()
            .join(", ");

        println!(
            "    │ {:<cat_len$} │ {:<id_len$} │ {:<ver_len$} │ {:<stat_len$} │ {:<targ_len$} │",
            p.category,
            p.id(),
            p.version,
            status_str,
            targets_str,
            cat_len = cat_w - 2,
            id_len = id_w - 2,
            ver_len = ver_w - 2,
            stat_len = stat_w - 2,
            targ_len = targ_w - 2
        );
    }
    println!("    {bot}");
}