use crate::shared::config::Config;
use directories_next::ProjectDirs;
use once_cell::sync::Lazy;

use std::{fs, path::PathBuf};

pub static DB_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let project_dirs =
        ProjectDirs::from("", "", "Shrike").expect("Failed to get project directories");
    let config = Config::new().expect("Failed to load configuration");

    let mut path = project_dirs.data_local_dir().to_path_buf();
    path.push(config.get_rpc_folder_name());
    path.push("shrike.db3");

    // Check if the parent directory exists and create it if necessary
    let parent = path.parent().expect("Failed to get db parent directory");
    if !parent.exists() {
        fs::create_dir_all(parent).expect("Failed to create db parent directory");
    }

    path
});
