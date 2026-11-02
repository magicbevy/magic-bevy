use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct PackageInfo {
    pub id: String,
    pub category: String,
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub targets: Vec<String>,
    pub path: PathBuf,
    pub author: String,
    pub description: String,
    pub has_banner: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    List,
}
