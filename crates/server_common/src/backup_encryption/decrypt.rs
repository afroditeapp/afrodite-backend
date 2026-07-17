//! Decryption for backup format-0 and format-1 data.

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use aes_gcm::{Aes128Gcm, KeyInit, aead::Aead};

/// Decrypt format-0 (single-chunk) backup data.
///
/// Input: `[0u8; 1B] [nonce; 12B] [ciphertext+tag; rest]`
fn decrypt_backup_data(key: &[u8; 16], data: &[u8]) -> Result<Vec<u8>, DecryptError> {
    let cipher = Aes128Gcm::new_from_slice(key).expect("valid 16-byte key");
    if data.len() <= 13 {
        return Err(DecryptError::NotEnoughData);
    }
    assert_eq!(data[0], 0u8, "not format-0 data");
    let nonce = &data[1..13];
    let ciphertext = &data[13..];
    cipher
        .decrypt(aes_gcm::Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| DecryptError::DecryptionFailed)
}

/// Streaming decryptor for backup files (both format-0 and format-1).
pub struct DecryptBackupDataStream {
    file: File,
    key: [u8; 16],
    first: bool,
    eof: bool,
}

#[derive(Debug)]
pub enum DecryptError {
    /// I/O error reading the file.
    Io(io::Error),
    /// AES-GCM decryption failed (wrong key, corrupted data).
    DecryptionFailed,
    NotEnoughData,
}

impl From<io::Error> for DecryptError {
    fn from(e: io::Error) -> Self {
        DecryptError::Io(e)
    }
}

impl DecryptBackupDataStream {
    /// Open a backup file for streaming decryption.
    pub fn open(path: &Path, key: [u8; 16]) -> Result<Self, io::Error> {
        Ok(Self {
            file: File::open(path)?,
            key,
            first: true,
            eof: false,
        })
    }

    /// Decrypt the next chunk. Returns `Ok(None)` when all chunks have been read.
    ///
    /// # Errors
    /// - `DecryptError::Io` if the file cannot be read.
    /// - `DecryptError::DecryptionFailed` if AES-GCM authentication fails.
    pub fn decrypt_next_chunk(&mut self) -> Result<Option<Vec<u8>>, DecryptError> {
        if self.eof {
            return Ok(None);
        }

        if self.first {
            // First chunk has a format byte prefix (1u8)
            let mut format_buf = [0u8; 1];
            if self.file.read_exact(&mut format_buf).is_err() {
                self.eof = true;
                return Ok(None);
            }
            if format_buf[0] == 0 {
                self.eof = true;
                let mut buffer = format_buf.to_vec();
                self.file.read_to_end(&mut buffer)?;
                return decrypt_backup_data(&self.key, &buffer).map(Some);
            } else {
                self.first = false;
            }
        }

        let mut size_buf = [0u8; 4];
        if self.file.read_exact(&mut size_buf).is_err() {
            self.eof = true;
            return Ok(None);
        }

        let chunk_size = u32::from_le_bytes(size_buf) as usize;

        let mut buf = vec![0u8; chunk_size];
        self.file.read_exact(&mut buf)?;

        let nonce = &buf[..12];
        let ciphertext = &buf[12..];

        let cipher = Aes128Gcm::new_from_slice(&self.key).expect("valid 16-byte key");
        let plaintext = cipher
            .decrypt(aes_gcm::Nonce::from_slice(nonce), ciphertext)
            .map_err(|_| DecryptError::DecryptionFailed)?;
        Ok(Some(plaintext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup_encryption::{encrypt_backup_data, encrypt_backup_data_stream};

    #[test]
    fn decrypt_format_0_roundtrip() {
        let key = [0xAB; 16];
        let plaintext = b"hello backup world";

        let encrypted = encrypt_backup_data(&key, plaintext);

        let dir = std::env::temp_dir();
        let path = dir.join("decrypt_fmt0_roundtrip.bin");
        std::fs::write(&path, &encrypted).unwrap();

        let mut stream = DecryptBackupDataStream::open(&path, key).unwrap();
        let result = stream
            .decrypt_next_chunk()
            .unwrap()
            .expect("should have one chunk");
        assert!(stream.decrypt_next_chunk().unwrap().is_none());
        assert_eq!(result, plaintext);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn decrypt_format_0_and_format_1_dispatch() {
        let key = [0xAB; 16];

        // format-0 data goes through the format-0 path in DecryptBackupDataStream
        let pt0 = b"format-zero data";
        let enc0 = encrypt_backup_data(&key, pt0);
        let dir = std::env::temp_dir();
        let path0 = dir.join("decrypt_dispatch_0.bin");
        std::fs::write(&path0, &enc0).unwrap();
        let mut s0 = DecryptBackupDataStream::open(&path0, key).unwrap();
        let got0 = s0.decrypt_next_chunk().unwrap().expect("fmt0 chunk");
        assert!(s0.decrypt_next_chunk().unwrap().is_none());
        assert_eq!(got0, pt0);
        let _ = std::fs::remove_file(&path0);

        // format-1 data goes through the chunk-by-chunk path
        let pt1 = b"format-one data";
        let enc1 = encrypt_backup_data_stream(&key, pt1, true);
        let path1 = dir.join("decrypt_dispatch_1.bin");
        std::fs::write(&path1, &enc1).unwrap();
        let mut s1 = DecryptBackupDataStream::open(&path1, key).unwrap();
        let got1 = s1.decrypt_next_chunk().unwrap().expect("fmt1 chunk");
        assert!(s1.decrypt_next_chunk().unwrap().is_none());
        assert_eq!(got1, pt1);
        let _ = std::fs::remove_file(&path1);
    }

    #[test]
    fn decrypt_stream_single_chunk() {
        let key = [0xCD; 16];
        let plaintext = b"single chunk stream";

        let encrypted = encrypt_backup_data_stream(&key, plaintext, true);

        let dir = std::env::temp_dir();
        let path = dir.join("decrypt_test_single.bin");
        std::fs::write(&path, &encrypted).unwrap();

        let mut stream = DecryptBackupDataStream::open(&path, key).unwrap();
        let result = stream
            .decrypt_next_chunk()
            .unwrap()
            .expect("should have one chunk");
        let done = stream.decrypt_next_chunk().unwrap();
        assert!(done.is_none());
        assert_eq!(result, plaintext);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn decrypt_stream_multiple_chunks() {
        let key = [0xEF; 16];
        let chunks: &[&[u8]] = &[b"first chunk data", b"second chunk!", b"third and last"];

        let mut all_encrypted = Vec::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let enc = encrypt_backup_data_stream(&key, chunk, i == 0);
            all_encrypted.extend_from_slice(&enc);
        }

        let dir = std::env::temp_dir();
        let path = dir.join("decrypt_test_multi.bin");
        std::fs::write(&path, &all_encrypted).unwrap();

        let mut stream = DecryptBackupDataStream::open(&path, key).unwrap();
        for expected in chunks {
            let got = stream
                .decrypt_next_chunk()
                .unwrap()
                .expect("should have chunk");
            assert_eq!(got, *expected);
        }
        assert!(stream.decrypt_next_chunk().unwrap().is_none());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn decrypt_stream_empty_input() {
        let key = [0x42; 16];
        let dir = std::env::temp_dir();
        let path = dir.join("decrypt_test_empty.bin");
        std::fs::write(&path, []).unwrap();

        let mut stream = DecryptBackupDataStream::open(&path, key).unwrap();
        assert!(stream.decrypt_next_chunk().unwrap().is_none());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn decrypt_stream_wrong_key_fails() {
        let good_key = [0x11; 16];
        let wrong_key = [0x22; 16];
        let plaintext = b"secret data";

        let encrypted = encrypt_backup_data_stream(&good_key, plaintext, true);

        let dir = std::env::temp_dir();
        let path = dir.join("decrypt_test_wrong_key.bin");
        std::fs::write(&path, &encrypted).unwrap();

        let mut stream = DecryptBackupDataStream::open(&path, wrong_key).unwrap();
        let result = stream.decrypt_next_chunk();
        assert!(matches!(result, Err(DecryptError::DecryptionFailed)));

        let _ = std::fs::remove_file(&path);
    }
}
