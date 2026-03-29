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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_get_dict_path() {
        let dir = Path::new("/tmp/test-config");
        let path = get_dict_path(dir);
        assert_eq!(path, Path::new("/tmp/test-config/ecdict-28.mdx"));
    }

    #[test]
    fn test_is_dict_downloaded_false() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        assert!(!is_dict_downloaded(dir.path()));
    }

    #[test]
    fn test_is_dict_downloaded_true() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let dict_path = get_dict_path(dir.path());
        fs::write(&dict_path, "dummy").expect("write should succeed");
        assert!(is_dict_downloaded(dir.path()));
    }
}
