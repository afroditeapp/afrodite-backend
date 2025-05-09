//! Message encrypting code from client

use std::{error::Error, fmt::Display};

use bstr::BStr;
use pgp::{
    crypto::{aead::AeadAlgorithm, hash::HashAlgorithm, sym::SymmetricKeyAlgorithm}, ser::Serialize, types::SecretKeyTrait, ArmorOptions, Deserializable, KeyType, Message, PlainSessionKey, SecretKeyParamsBuilder, SignedPublicKey, SignedSecretKey, SubkeyParamsBuilder
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
    PublicKeyReadFromString = 30,
    PublicKeyToBytes = 31,
    PrivateKeyReadFromString = 40,
    SignData = 50,
    SignDataToBytes = 51,
    UnwrapSignedMessageFromBytes = 60,
    UnwrapSignedMessageGetContent = 61,
    UnwrapSignedMessageNoContent = 62,
    VerifySignedMessageFromBytes = 70,
    VerifySignedMessageVerify = 71,
    VerifySignedMessageGetContent = 72,
    VerifySignedMessageNoContent = 73,
    VerifySignedMessageParsePublicKey = 74,
    DecryptWithKeyFromBytes = 80,
    DecryptWithKeyDecrypt = 81,
    DecryptWithKeyNotEncrypted = 82,
    DecryptWithKeyToBytes = 83,
}

impl Display for MessageEncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for MessageEncryptionError {}

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
        .sign(OsRng, &my_private_key, String::new, HashAlgorithm::SHA2_256)
        .map_err(|_| MessageEncryptionError::EncryptDataSign)?
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
        .to_bytes()
        .map_err(|_| MessageEncryptionError::EncryptDataToBytes)?;

    Ok(armored_message)
}

pub fn generate_keys(
    primary_user_id: String,
) -> Result<GeneratedKeys, MessageEncryptionError>  {
    let params = SecretKeyParamsBuilder::default()
        .key_type(KeyType::Ed25519)
        .can_encrypt(false)
        .can_certify(false)
        .can_sign(true)
        .primary_user_id(primary_user_id)
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
    /// ASCII armored PGP private key
    pub private: String,
    /// ASCII armored PGP public key
    pub public: String,
}

impl GeneratedKeys {
    pub fn public_key_bytes(&self) -> Result<Vec<u8>, MessageEncryptionError> {
        let (public_key, _) = SignedPublicKey::from_string(&self.public)
            .map_err(|_| MessageEncryptionError::PublicKeyReadFromString)?;
        public_key.to_bytes()
            .map_err(|_| MessageEncryptionError::PublicKeyToBytes)
    }

    pub fn to_parsed_keys(&self) -> Result<ParsedKeys, MessageEncryptionError> {
        let (public, _) = SignedPublicKey::from_string(&self.public)
            .map_err(|_| MessageEncryptionError::PublicKeyReadFromString)?;
        let (private, _) = SignedSecretKey::from_string(&self.private)
            .map_err(|_| MessageEncryptionError::PrivateKeyReadFromString)?;

        Ok(ParsedKeys { private, public })
    }
}

pub struct ParsedKeys {
    private: SignedSecretKey,
    public: SignedPublicKey,
}

impl ParsedKeys {
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, MessageEncryptionError> {
        let empty_file_name: &BStr = b"".into();
        let message = pgp::message::Message::new_literal_bytes(empty_file_name, data)
            .sign(OsRng, &self.private, String::new, HashAlgorithm::SHA2_256)
            .map_err(|_| MessageEncryptionError::SignData)?;

        message.to_bytes()
            .map_err(|_| MessageEncryptionError::SignDataToBytes)
    }

    pub fn verify_signed_message_and_extract_data(&self, data: &[u8]) -> Result<Vec<u8>, MessageEncryptionError> {
        let message = pgp::message::Message::from_bytes(data)
            .map_err(|_| MessageEncryptionError::VerifySignedMessageFromBytes)?;

        message.verify(&self.public)
            .map_err(|_| MessageEncryptionError::VerifySignedMessageVerify)?;

        let data = message.get_content()
            .map_err(|_| MessageEncryptionError::VerifySignedMessageGetContent)?
            .ok_or(MessageEncryptionError::VerifySignedMessageNoContent)?;

        Ok(data)
    }
}

pub fn unwrap_signed_binary_message(data: &[u8]) -> Result<Vec<u8>, MessageEncryptionError> {
    let message = pgp::message::Message::from_bytes(data)
        .map_err(|_| MessageEncryptionError::UnwrapSignedMessageFromBytes)?;
    message.get_content()
        .map_err(|_| MessageEncryptionError::UnwrapSignedMessageGetContent)?
        .ok_or(MessageEncryptionError::UnwrapSignedMessageNoContent)
}

pub fn verify_signed_binary_message(data: &[u8], pgp_public_key: &[u8]) -> Result<Vec<u8>, MessageEncryptionError> {
    let public_key = SignedPublicKey::from_bytes(pgp_public_key)
        .map_err(|_| MessageEncryptionError::VerifySignedMessageParsePublicKey)?;
    let message = pgp::message::Message::from_bytes(data)
        .map_err(|_| MessageEncryptionError::VerifySignedMessageFromBytes)?;

    message.verify(&public_key)
        .map_err(|_| MessageEncryptionError::VerifySignedMessageVerify)?;

    let data = message.get_content()
        .map_err(|_| MessageEncryptionError::VerifySignedMessageGetContent)?
        .ok_or(MessageEncryptionError::VerifySignedMessageNoContent)?;

    Ok(data)
}

pub fn decrypt_binary_message(data: &[u8], key: &[u8]) -> Result<Vec<u8>, MessageEncryptionError> {
    let message = pgp::message::Message::from_bytes(data)
        .map_err(|_| MessageEncryptionError::DecryptWithKeyFromBytes)?;

    let key = PlainSessionKey::V6 { key: key.to_vec() };

    if let Message::Encrypted { edata, .. } = message {
        edata.decrypt(key)
            .map_err(|_| MessageEncryptionError::DecryptWithKeyDecrypt)?
            .to_bytes()
            .map_err(|_| MessageEncryptionError::DecryptWithKeyToBytes)
    } else {
        Err(MessageEncryptionError::DecryptWithKeyNotEncrypted)
    }
}
