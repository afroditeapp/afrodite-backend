use aes_gcm::{AeadCore, Aes128Gcm, KeyInit, aead::Aead};
use rand::rngs::OsRng;

/// Encrypt format-0 (single-chunk backups).
///
/// Returns: 0 byte + 12-byte nonce + ciphertext+tag.
pub fn encrypt_backup_data(key: &[u8; 16], plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes128Gcm::new_from_slice(key).expect("valid 16-byte key");
    let nonce = Aes128Gcm::generate_nonce(OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("AES-128-GCM encryption should not fail with valid key");

    let mut result = Vec::with_capacity(1 + 12 + ciphertext.len());
    result.extend_from_slice(&[0u8]);
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);
    result
}

/// Encrypt format-1 (multi-chunk backups).
///
/// When `first_chunk` is true, returns: format byte `1` (1B) + chunk size (4B LE) + 12-byte nonce + ciphertext+tag.
/// When `first_chunk` is false, returns: chunk size (4B LE) + 12-byte nonce + ciphertext+tag.
pub fn encrypt_backup_data_stream(key: &[u8; 16], plaintext: &[u8], first_chunk: bool) -> Vec<u8> {
    let cipher = Aes128Gcm::new_from_slice(key).expect("valid 16-byte key");
    let nonce = Aes128Gcm::generate_nonce(OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("AES-128-GCM encryption should not fail with valid key");

    let mut result = if first_chunk {
        let mut result = Vec::with_capacity(1 + 4 + 12 + ciphertext.len());
        result.push(1u8);
        result
    } else {
        Vec::with_capacity(4 + 12 + ciphertext.len())
    };

    let chunk_size: u32 = 12 + ciphertext.len() as u32;
    result.extend_from_slice(&chunk_size.to_le_bytes());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);
    result
}

pub mod decrypt;

#[cfg(test)]
mod tests {
    use aes_gcm::aead::Aead;

    use super::*;

    const BACKUP_NONCE_SIZE: usize = 12;

    #[test]
    fn encrypt_format_0() {
        let key = [0xAB; 16];
        let plaintext = b"content file";

        let encrypted = encrypt_backup_data(&key, plaintext);

        assert_eq!(encrypted[0], 0); // format byte
        assert_eq!(encrypted.len(), 1 + 12 + 16 + plaintext.len()); // fmt + nonce + tag + plaintext
        let nonce: &[u8; 12] = encrypted[1..13].try_into().unwrap();
        let ciphertext = &encrypted[13..];

        let cipher = Aes128Gcm::new_from_slice(&key).unwrap();
        let decrypted = cipher
            .decrypt(aes_gcm::Nonce::from_slice(nonce), ciphertext)
            .expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_format_1() {
        let key = [0xAB; 16];
        let plaintext = b"database backup chunk";

        let encrypted = encrypt_backup_data_stream(&key, plaintext, true);

        assert_eq!(encrypted[0], 1); // format byte
        let chunk_size = u32::from_le_bytes(encrypted[1..5].try_into().unwrap());
        assert_eq!(chunk_size as usize, encrypted.len() - 5);
        assert_eq!(encrypted[5..17].len(), BACKUP_NONCE_SIZE);
        assert!(encrypted.len() > 17);

        let cipher = Aes128Gcm::new_from_slice(&key).unwrap();
        let nonce: &[u8; 12] = encrypted[5..17].try_into().unwrap();
        let decrypted = cipher
            .decrypt(aes_gcm::Nonce::from_slice(nonce), &encrypted[17..])
            .expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_subsequent_chunk() {
        let key = [0xAB; 16];
        let plaintext = b"another chunk";

        let encrypted = encrypt_backup_data_stream(&key, plaintext, false);

        let chunk_size = u32::from_le_bytes(encrypted[..4].try_into().unwrap());
        assert_eq!(chunk_size as usize, encrypted.len() - 4);
        let nonce: &[u8; 12] = encrypted[4..16].try_into().unwrap();
        let ciphertext = &encrypted[16..];

        let cipher = Aes128Gcm::new_from_slice(&key).unwrap();
        let decrypted = cipher
            .decrypt(aes_gcm::Nonce::from_slice(nonce), ciphertext)
            .expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }
}
