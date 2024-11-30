//! Message encrypting code from client

use bstr::BStr;
use pgp::{
    crypto::hash::HashAlgorithm, ser::Serialize, Deserializable, Message, SignedPublicKey,
    SignedSecretKey,
};
use rand::rngs::OsRng;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MessageEncryptionError {
    GenerateKeysPrivateKeyParams = 1,
    GenerateKeysPrivateKeyGenerate = 2,
    GenerateKeysPrivateKeySign = 3,
    GenerateKeysPrivateKeyArmor = 4,
    GenerateKeysPrivateKeyNullDetected = 5,
    GenerateKeysPublicKeySign = 6,
    GenerateKeysPublicKeyArmor = 7,
    GenerateKeysPublicKeyNullDetected = 8,
    GenerateKeysPrivateKeySubKeyParams = 9,
    EncryptDataPrivateKeyParse = 10,
    EncryptDataPublicKeyParse = 11,
    EncryptDataEncrypt = 12,
    EncryptDataSign = 13,
    EncryptDataToBytes = 14,
    EncryptDataPublicSubkeyMissing = 15,
    EncryptDataEncryptedMessageLenTooLarge = 16,
    EncryptDataEncryptedMessageCapacityTooLarge = 17,
    DecryptDataPrivateKeyParse = 20,
    DecryptDataPublicKeyParse = 21,
    DecryptDataMessageParse = 22,
    DecryptDataVerify = 23,
    DecryptDataDecrypt = 24,
    DecryptDataDataNotFound = 25,
    DecryptDataDecryptedMessageLenTooLarge = 26,
    DecryptDataDecryptedMessageCapacityTooLarge = 27,
}

pub fn encrypt_data(
    // The sender private key can be used for signing the message
    data_sender_armored_private_key: &str,
    data_receiver_armored_public_key: &str,
    data: &[u8],
) -> Result<Vec<u8>, MessageEncryptionError> {
    let (my_private_key, _) = SignedSecretKey::from_string(data_sender_armored_private_key)
        .map_err(|_| MessageEncryptionError::EncryptDataPrivateKeyParse)?;
    let (other_person_public_key, _) =
        SignedPublicKey::from_string(data_receiver_armored_public_key)
            .map_err(|_| MessageEncryptionError::EncryptDataPublicKeyParse)?;

    let empty_file_name: &BStr = b"".into();

    let encryption_public_subkey = other_person_public_key
        .public_subkeys
        .first()
        .ok_or(MessageEncryptionError::EncryptDataPublicSubkeyMissing)?;

    let armored_message = Message::new_literal_bytes(empty_file_name, data)
        // Compression is not done for now as this library does not
        // have possibility to limit decompressed data size.
        // If the data would be compressed, then denial of service attacks
        // would be possible.
        .encrypt_to_keys(
            &mut OsRng,
            pgp::crypto::sym::SymmetricKeyAlgorithm::AES128,
            &[encryption_public_subkey],
        )
        .map_err(|_| MessageEncryptionError::EncryptDataEncrypt)?
        .sign(&my_private_key, String::new, HashAlgorithm::SHA2_256)
        .map_err(|_| MessageEncryptionError::EncryptDataSign)?
        .to_bytes()
        .map_err(|_| MessageEncryptionError::EncryptDataToBytes)?;

    Ok(armored_message)
}
