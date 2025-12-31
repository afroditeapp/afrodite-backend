use std::path::{Path, PathBuf};

pub fn abs_path_for_directory_or_file_which_might_not_exists(
    path: impl AsRef<Path>,
) -> Result<PathBuf, std::io::Error> {
    let path = path.as_ref();
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}
