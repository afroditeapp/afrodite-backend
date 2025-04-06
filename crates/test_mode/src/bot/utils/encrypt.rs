//! Message encrypting code from client

use bstr::BStr;
use pgp::{
    crypto::{aead::AeadAlgorithm, hash::HashAlgorithm, sym::SymmetricKeyAlgorithm}, ser::Serialize, types::SecretKeyTrait, ArmorOptions, Deserializable, KeyType, Message, SecretKeyParamsBuilder, SignedPublicKey, SignedSecretKey, SubkeyParamsBuilder
};
use rand::rngs::OsRng;
use smallvec::smallvec;

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
    data_receive_public_key: Vec<u8>,
    data: &[u8],
) -> Result<Vec<u8>, MessageEncryptionError> {
    let (my_private_key, _) = SignedSecretKey::from_string(data_sender_armored_private_key)
        .map_err(|_| MessageEncryptionError::EncryptDataPrivateKeyParse)?;
    let other_person_public_key =
        SignedPublicKey::from_bytes(data_receive_public_key.as_slice())
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
        .encrypt_to_keys_seipdv2(
            OsRng,
            SymmetricKeyAlgorithm::AES128,
            AeadAlgorithm::Gcm,
            // Use max chunk size as message size is small
            // and streaming decryption is not needed.
            16,
            &[encryption_public_subkey],
        )
        .map_err(|_| MessageEncryptionError::EncryptDataEncrypt)?
        .sign(OsRng, &my_private_key, String::new, HashAlgorithm::SHA2_256)
        .map_err(|_| MessageEncryptionError::EncryptDataSign)?
        .to_bytes()
        .map_err(|_| MessageEncryptionError::EncryptDataToBytes)?;

    Ok(armored_message)
}

pub fn generate_keys(
    account_id: String,
) -> Result<GeneratedKeys, MessageEncryptionError>  {
    let params = SecretKeyParamsBuilder::default()
        .key_type(KeyType::Ed25519)
        .can_encrypt(false)
        .can_certify(false)
        .can_sign(true)
        .primary_user_id(account_id)
        .preferred_symmetric_algorithms(smallvec![
            SymmetricKeyAlgorithm::AES128,
        ])
        .preferred_hash_algorithms(smallvec![
            HashAlgorithm::SHA2_256,
        ])
        .preferred_compression_algorithms(smallvec![])
        .subkey(
            SubkeyParamsBuilder::default()
                .key_type(KeyType::X25519)
                .can_authenticate(false)
                .can_certify(false)
                .can_encrypt(true)
                .can_sign(false)
                .build()
                .map_err(|_| MessageEncryptionError::GenerateKeysPrivateKeySubKeyParams)?
        )
        .build()
        .map_err(|_| MessageEncryptionError::GenerateKeysPrivateKeyParams)?;
    let private_key = params
        .generate(OsRng)
        .map_err(|_| MessageEncryptionError::GenerateKeysPrivateKeyGenerate)?;
    let signed_private_key = private_key
        .sign(OsRng, String::new)
        .map_err(|_| MessageEncryptionError::GenerateKeysPrivateKeySign)?;
    let private = signed_private_key
        .to_armored_string(ArmorOptions::default())
        .map_err(|_| MessageEncryptionError::GenerateKeysPrivateKeyArmor)?;

    let signed_public_key = signed_private_key
        .public_key()
        .sign(OsRng, &signed_private_key, String::new)
        .map_err(|_| MessageEncryptionError::GenerateKeysPublicKeySign)?;
    let public = signed_public_key
        .to_armored_string(ArmorOptions::default())
        .map_err(|_| MessageEncryptionError::GenerateKeysPublicKeyArmor)?;

    Ok(GeneratedKeys {
        private,
        public,
    })
}

pub struct GeneratedKeys {
    pub private: String,
    pub public: String,
}
