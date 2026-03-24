pub mod downloader;
pub mod reader;

use std::path::{Path, PathBuf};

pub const DICT_URL: &str = "https://github.com/skywind3000/ECDICT/releases/download/1.0.28/ecdict-mdx-28.zip";
pub const DICT_FILENAME: &str = "ecdict-28.mdx";
pub const DICT_ZIP_FILENAME: &str = "ecdict-mdx-28.zip";
pub const DICT_FILE_SIZE: u64 = 93_200_000; // ~93.2 MB

pub fn get_dict_path(config_dir: &Path) -> PathBuf {
    config_dir.join(DICT_FILENAME)
}

pub fn is_dict_downloaded(config_dir: &Path) -> bool {
    get_dict_path(config_dir).exists()
}
