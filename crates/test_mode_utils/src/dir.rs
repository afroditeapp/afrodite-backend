use std::path::PathBuf;

use config::args::TestMode;

pub struct DataDirUtils;

impl DataDirUtils {
    pub fn create_data_dir_if_needed(config: &TestMode) -> PathBuf {
        let Some(dir) = config.data_dir.clone() else {
            panic!("Test mode data dir is not configured");
        };
        if !dir.exists() {
            std::fs::create_dir_all(&dir).unwrap();
        }
        dir
    }
}
