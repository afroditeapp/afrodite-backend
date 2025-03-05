use std::path::Path;

use error_stack::{Result, ResultExt};
use tokio::io::AsyncWriteExt;

#[derive(thiserror::Error, Debug)]
pub enum OverwriteFileError {
    // File IO errors
    #[error("File open failed")]
    IoFileOpen,
    #[error("File writing failed")]
    IoFileWrite,
    #[error("File flushing failed")]
    IoFileFlush,
    #[error("File sync failed")]
    IoFileSync,
    #[error("File remove failed")]
    IoFileRemove,
    #[error("Getting file metadata failed")]
    IoFileMetadata,

    #[error("File overwriteing failed")]
    FileOverwritingFailed,
}


pub async fn overwrite_and_remove_if_exists(path: &Path) -> Result<(), OverwriteFileError> {
    if !path.exists() {
        return Ok(());
    }

    overwrite_file(path).await?;

    tokio::fs::remove_file(path)
        .await
        .change_context(OverwriteFileError::IoFileRemove)
}

/// This is enough for Ext4 file system as data=journal is
/// not the default.
///
/// https://manpages.ubuntu.com/manpages/focal/man1/shred.1.html
/// https://www.kernel.org/doc/Documentation/filesystems/ext4.txt
async fn overwrite_file(path: &Path) -> Result<(), OverwriteFileError> {
    let mut file = tokio::fs::File::options().write(true).open(&path)
        .await
        .change_context(OverwriteFileError::IoFileOpen)?;

    let data = file.metadata()
        .await
        .change_context(OverwriteFileError::IoFileMetadata)?;
    let file_len: usize = TryInto::<usize>::try_into(data.len())
        .change_context(OverwriteFileError::FileOverwritingFailed)?;

    for zeros in buffered_zero_iter(file_len) {
        file.write_all(zeros)
            .await
            .change_context(OverwriteFileError::IoFileWrite)?;
    }

    file
        .flush()
        .await
        .change_context(OverwriteFileError::IoFileFlush)?;

    file
        .sync_all()
        .await
        .change_context(OverwriteFileError::IoFileSync)?;

    Ok(())
}

const ZERO_ITER_CHUNK_SIZE: usize = 4096;

fn buffered_zero_iter(
    bytes: usize,
) -> impl Iterator<Item=&'static [u8]> {
    const ZERO_BUFFER: [u8; ZERO_ITER_CHUNK_SIZE] = [0; ZERO_ITER_CHUNK_SIZE];
    let iter = std::iter::repeat(ZERO_BUFFER.as_slice())
        .take(bytes/ZERO_ITER_CHUNK_SIZE);
    let remaining_bytes =
        [&ZERO_BUFFER[..(bytes % ZERO_ITER_CHUNK_SIZE)]].into_iter().take_while(|v| !v.is_empty());
    iter
        .chain(remaining_bytes)
}

#[cfg(test)]
mod test {
    use super::{buffered_zero_iter, ZERO_ITER_CHUNK_SIZE};

    fn assert_eq_iter(
        mut value: impl Iterator<Item=&'static [u8]>,
        expected: impl IntoIterator<Item=&'static [u8]>,
    ) {
        let mut expected = expected.into_iter();
        loop {
            match (value.next(), expected.next()) {
                (None, None) => break,
                (Some(value), Some(expected)) if value == expected => continue,
                _ => panic!("Values differ"),
            }
        }
    }

    #[test]
    fn zero_iter_empty() {
        const SIZE: usize = 0;
        assert_eq_iter(
            buffered_zero_iter(SIZE),
            [],
        )
    }

    #[test]
    fn zero_iter_less_than_buffer_size() {
        const SIZE: usize = ZERO_ITER_CHUNK_SIZE - 1;
        assert_eq_iter(
            buffered_zero_iter(SIZE),
            [[0u8; SIZE].as_slice()],
        )
    }

    #[test]
    fn zero_iter_equal_size_as_buffer_size() {
        const SIZE: usize = ZERO_ITER_CHUNK_SIZE;
        assert_eq_iter(
            buffered_zero_iter(SIZE),
            [[0u8; SIZE].as_slice()],
        );
    }

    #[test]
    fn zero_iter_larger_size_than_buffer_size() {
        const SIZE: usize = ZERO_ITER_CHUNK_SIZE + 1;
        assert_eq_iter(
            buffered_zero_iter(SIZE),
            [[0u8; ZERO_ITER_CHUNK_SIZE].as_slice(), [0u8; 1].as_slice()],
        );
    }
}
