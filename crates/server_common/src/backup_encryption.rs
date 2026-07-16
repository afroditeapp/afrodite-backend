use aes_gcm::{AeadCore, Aes128Gcm, KeyInit, aead::Aead};
use rand::rngs::OsRng;

/// Magic bytes at start of every encrypted backup file
const BACKUP_FILE_MAGIC: &[u8] = b"aesgcm128";

/// Encrypt plaintext with AES-128-GCM using the given 16-byte key.
///
/// Returns: `"aesgcm128"` (9 ASCII bytes) + 12-byte nonce + ciphertext+tag.
///
/// # Panics
/// Panics if `key` is not exactly 16 bytes (enforced by caller validation).
pub fn encrypt_backup_data(key: &[u8; 16], plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes128Gcm::new_from_slice(key).expect("valid 16-byte key");
    let nonce = Aes128Gcm::generate_nonce(OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("AES-128-GCM encryption should not fail with valid key");

    let mut result = Vec::with_capacity(BACKUP_FILE_MAGIC.len() + 12 + ciphertext.len());
    result.extend_from_slice(BACKUP_FILE_MAGIC);
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);
    result
}

#[cfg(test)]
mod tests {
    use aes_gcm::aead::Aead;

    use super::*;

    const BACKUP_NONCE_SIZE: usize = 12;

    #[test]
    fn encrypt_produces_valid_format() {
        let key = [0xAB; 16];
        let plaintext = b"Hello, backup!";

        let encrypted = encrypt_backup_data(&key, plaintext);

        assert_eq!(&encrypted[..9], b"aesgcm128");
        assert_eq!(encrypted[9..21].len(), BACKUP_NONCE_SIZE);
        assert!(encrypted.len() > 21);

        // Round-trip decrypt
        let cipher = Aes128Gcm::new_from_slice(&key).unwrap();
        let nonce: &[u8; 12] = encrypted[9..21].try_into().unwrap();
        let decrypted = cipher
            .decrypt(aes_gcm::Nonce::from_slice(nonce), &encrypted[21..])
            .expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }
}
