use std::{
    fs::File,
    path::{Path, PathBuf},
};

use error_stack::{Result, ResultExt, report};
use flate2::read::GzDecoder;
use manager_config::file::SimplePatternPath;
use tar::Archive;
use tracing::info;

use super::UpdateError;

pub async fn extract_backend_binary(
    archive: PathBuf,
    archive_file_path: SimplePatternPath,
    target: PathBuf,
) -> Result<(), UpdateError> {
    tokio::task::spawn_blocking(move || {
        let path = find_backend_binary_from_archive_sync(&archive, archive_file_path)?;
        extract_backend_binary_sync(&archive, path, target)
    })
    .await
    .change_context(UpdateError::Archive)?
}

fn find_backend_binary_from_archive_sync(
    archive: &Path,
    archive_file_path: SimplePatternPath,
) -> Result<PathBuf, UpdateError> {
    let file = File::open(archive).change_context(UpdateError::Archive)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let entries = archive.entries().change_context(UpdateError::Archive)?;

    let mut found_files = vec![];
    let mut matching_file: Option<(_, String)> = None;
    for e in entries {
        let e = e.change_context(UpdateError::Archive)?;
        if !e.header().entry_type().is_file() {
            continue;
        }

        let path = e.path().change_context(UpdateError::Archive)?;
        let path_string = path.to_string_lossy().to_string();
        found_files.push(path_string.clone());

        let matcher = FileMatcher {
            wanted: &archive_file_path,
            archive_path: &path,
        };

        if matcher.is_match() {
            if let Some((_, previous_match)) = &matching_file {
                return Err(report!(UpdateError::ArchiveMultipleMatchingFiles))
                    .attach_printable(previous_match.clone())
                    .attach_printable(path_string);
            } else {
                matching_file = Some((path.to_path_buf(), path_string));
            }
        }
    }

    if let Some((path, _)) = matching_file {
        Ok(path)
    } else {
        Err(report!(UpdateError::ArchiveBackendNotFound))
            .attach_printable(format!("Available files: {found_files:#?}"))
    }
}

fn extract_backend_binary_sync(
    archive: &Path,
    archive_file_path: PathBuf,
    target: PathBuf,
) -> Result<(), UpdateError> {
    let file = File::open(archive).change_context(UpdateError::Archive)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let entries = archive.entries().change_context(UpdateError::Archive)?;

    for e in entries {
        let mut e = e.change_context(UpdateError::Archive)?;
        if !e.header().entry_type().is_file() {
            continue;
        }

        let path = e.path().change_context(UpdateError::Archive)?;
        if path != archive_file_path {
            continue;
        }

        info!("Extracting file {}", path.to_string_lossy());

        let mut target_file = File::create(target).change_context(UpdateError::Archive)?;
        std::io::copy(&mut e, &mut target_file).change_context(UpdateError::Archive)?;

        return Ok(());
    }

    Err(report!(UpdateError::ArchiveBackendNotFound))
}

struct FileMatcher<'a> {
    wanted: &'a SimplePatternPath,
    archive_path: &'a Path,
}

impl FileMatcher<'_> {
    fn is_match(&self) -> bool {
        if self.wanted.0.iter().count() != self.archive_path.iter().count() {
            return false;
        }

        for (wanted, path_component) in self.wanted.0.iter().zip(self.archive_path) {
            if wanted == "*" {
                continue;
            } else if wanted != path_component {
                return false;
            }
        }

        true
    }
}
