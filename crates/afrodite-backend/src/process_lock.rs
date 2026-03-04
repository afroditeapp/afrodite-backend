use std::{
    fs::{File, OpenOptions, create_dir_all},
    io,
    path::Path,
};

const SERVER_LOCK_FILE_NAME: &str = "server.lock";

pub struct ServerProcessLock {
    _file: File,
}

fn open_lock_file(path: &Path) -> io::Result<File> {
    if let Some(parent_dir) = path.parent() {
        create_dir_all(parent_dir)?;
    }

    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
}

/// Creates data directory if it doesn't exist.
pub fn acquire_server_lock(data_dir: &Path) -> Result<ServerProcessLock, String> {
    let path = data_dir.join(SERVER_LOCK_FILE_NAME);
    let file = open_lock_file(&path)
        .map_err(|e| format!("Failed to open server lock file '{}': {e}", path.display()))?;

    file.try_lock().map_err(|e| match e {
        std::fs::TryLockError::WouldBlock => format!(
            "Refusing to continue because server lock is already held: {}",
            path.display()
        ),
        std::fs::TryLockError::Error(io_error) => format!(
            "Failed to lock server lock file '{}': {}",
            path.display(),
            io_error
        ),
    })?;

    Ok(ServerProcessLock { _file: file })
}
