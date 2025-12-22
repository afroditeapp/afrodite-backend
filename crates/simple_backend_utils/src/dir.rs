use std::path::{Path, PathBuf};

pub fn abs_path_for_directory_which_might_not_exists(
    path: &Path,
) -> Result<PathBuf, std::io::Error> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}
