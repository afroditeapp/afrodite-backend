//! Message encrypting code from client

use std::{error::Error, fmt::Display, io::Read};

use pgp::{
    bytes::Bytes,
    composed::{
        ArmorOptions, Deserializable, KeyType, Message, MessageBuilder, PlainSessionKey,
        SecretKeyParamsBuilder, SignedPublicKey, SignedSecretKey, SubkeyParamsBuilder,
    },
    crypto::{
        aead::{AeadAlgorithm, ChunkSize},
        hash::HashAlgorithm,
        sym::SymmetricKeyAlgorithm,
    },
    ser::Serialize,
    types::Password,
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
    GenerateKeysPublicKeyArmor = 5,
    GenerateKeysPrivateKeySubKeyParams = 6,
    EncryptDataPrivateKeyParse = 10,
    EncryptDataPublicKeyParse = 11,
    EncryptDataEncrypt = 12,
    EncryptDataToWriter = 13,
    EncryptDataPublicSubkeyMissing = 14,
    PublicKeyReadFromString = 30,
    PublicKeyToBytes = 31,
    PrivateKeyReadFromString = 40,
    SignDataToWriter = 50,
    UnwrapSignedMessageFromBytes = 60,
    UnwrapSignedMessageAsDataVec = 61,
    VerifySignedMessageFromBytes = 70,
    VerifySignedMessageVerify = 71,
    VerifySignedMessageAsDataVec = 72,
    VerifySignedMessageParsePublicKey = 73,
    DecryptWithKeyFromBytes = 80,
    DecryptWithKeyDecrypt = 81,
    DecryptWithKeyNotEncrypted = 82,
    DecryptWithKeyReadToEnd = 83,
}

impl Display for MessageEncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for MessageEncryptionError {}

pub fn encrypt_data(
    // The sender private key can be used for signing the message
    data_sender_armored_private_key: &str,
    data_receive_public_key: Vec<u8>,
    data: impl Into<pgp::bytes::Bytes>,
) -> Result<Vec<u8>, MessageEncryptionError> {
    let (my_private_key, _) = SignedSecretKey::from_string(data_sender_armored_private_key)
        .map_err(|_| MessageEncryptionError::EncryptDataPrivateKeyParse)?;
    let other_person_public_key = SignedPublicKey::from_bytes(data_receive_public_key.as_slice())
        .map_err(|_| MessageEncryptionError::EncryptDataPublicKeyParse)?;

    let encryption_public_subkey = other_person_public_key
        .public_subkeys
        .first()
        .ok_or(MessageEncryptionError::EncryptDataPublicSubkeyMissing)?;

    let mut output = vec![];

    let mut builder = MessageBuilder::from_bytes("", Into::<Bytes>::into(data));
    // Compression is not done for now as this library does not
    // have possibility to limit decompressed data size.
    // If the data would be compressed, then denial of service attacks
    // would be possible.
    builder.sign(
        &my_private_key.primary_key,
        Password::empty(),
        HashAlgorithm::Sha256,
    );
    let mut builder = builder.seipd_v2(
        OsRng,
        SymmetricKeyAlgorithm::AES128,
        AeadAlgorithm::Gcm,
        // Use max chunk size as message size is small
        // and streaming decryption is not needed.
        ChunkSize::C4MiB,
    );
    builder
        .encrypt_to_key(OsRng, encryption_public_subkey)
        .map_err(|_| MessageEncryptionError::EncryptDataEncrypt)?;
    builder
        .to_writer(OsRng, &mut output)
        .map_err(|_| MessageEncryptionError::EncryptDataToWriter)?;

    Ok(output)
}

pub fn generate_keys(primary_user_id: String) -> Result<GeneratedKeys, MessageEncryptionError> {
    let params = SecretKeyParamsBuilder::default()
        .key_type(KeyType::Ed25519)
        .can_encrypt(false)
        .can_certify(false)
        .can_sign(true)
        .primary_user_id(primary_user_id)
        .preferred_symmetric_algorithms(smallvec![SymmetricKeyAlgorithm::AES128,])
        .preferred_hash_algorithms(smallvec![HashAlgorithm::Sha256])
        .preferred_compression_algorithms(smallvec![])
        .subkey(
            SubkeyParamsBuilder::default()
                .key_type(KeyType::X25519)
                .can_authenticate(false)
                .can_encrypt(true)
                .can_sign(false)
                .build()
                .map_err(|_| MessageEncryptionError::GenerateKeysPrivateKeySubKeyParams)?,
        )
        .build()
        .map_err(|_| MessageEncryptionError::GenerateKeysPrivateKeyParams)?;
    let private_key = params
        .generate(OsRng)
        .map_err(|_| MessageEncryptionError::GenerateKeysPrivateKeyGenerate)?;
    let signed_private_key = private_key
        .sign(OsRng, &Password::empty())
        .map_err(|_| MessageEncryptionError::GenerateKeysPrivateKeySign)?;
    let private = signed_private_key
        .to_armored_string(ArmorOptions::default())
        .map_err(|_| MessageEncryptionError::GenerateKeysPrivateKeyArmor)?;

    let signed_public_key = signed_private_key.signed_public_key();
    let public = signed_public_key
        .to_armored_string(ArmorOptions::default())
        .map_err(|_| MessageEncryptionError::GenerateKeysPublicKeyArmor)?;

    Ok(GeneratedKeys { private, public })
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
        public_key
            .to_bytes()
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
    pub fn sign(
        &self,
        data: impl Into<pgp::bytes::Bytes>,
    ) -> Result<Vec<u8>, MessageEncryptionError> {
        let mut output = vec![];

        let mut builder = MessageBuilder::from_bytes("", data);
        builder.sign(
            &self.private.primary_key,
            Password::empty(),
            HashAlgorithm::Sha256,
        );

        builder
            .to_writer(OsRng, &mut output)
            .map_err(|_| MessageEncryptionError::SignDataToWriter)?;

        Ok(output)
    }

    pub fn verify_signed_message_and_extract_data(
        &self,
        data: &[u8],
    ) -> Result<Vec<u8>, MessageEncryptionError> {
        let mut message = Message::from_bytes(data)
            .map_err(|_| MessageEncryptionError::VerifySignedMessageFromBytes)?;

        let output = message
            .as_data_vec()
            .map_err(|_| MessageEncryptionError::VerifySignedMessageAsDataVec)?;

        message
            .verify_read(&self.public)
            .map_err(|_| MessageEncryptionError::VerifySignedMessageVerify)?;

        Ok(output)
    }
}

pub fn unwrap_signed_binary_message(data: &[u8]) -> Result<Vec<u8>, MessageEncryptionError> {
    Message::from_bytes(data)
        .map_err(|_| MessageEncryptionError::UnwrapSignedMessageFromBytes)?
        .as_data_vec()
        .map_err(|_| MessageEncryptionError::UnwrapSignedMessageAsDataVec)
}

pub fn verify_signed_binary_message(
    data: &[u8],
    pgp_public_key: &[u8],
) -> Result<Vec<u8>, MessageEncryptionError> {
    let public_key = SignedPublicKey::from_bytes(pgp_public_key)
        .map_err(|_| MessageEncryptionError::VerifySignedMessageParsePublicKey)?;
    let mut message = Message::from_bytes(data)
        .map_err(|_| MessageEncryptionError::VerifySignedMessageFromBytes)?;

    let output = message
        .as_data_vec()
        .map_err(|_| MessageEncryptionError::VerifySignedMessageAsDataVec)?;

    message
        .verify_read(&public_key)
        .map_err(|_| MessageEncryptionError::VerifySignedMessageVerify)?;

    Ok(output)
}

pub fn decrypt_binary_message(data: &[u8], key: &[u8]) -> Result<Vec<u8>, MessageEncryptionError> {
    let mut message =
        Message::from_bytes(data).map_err(|_| MessageEncryptionError::DecryptWithKeyFromBytes)?;

    let key = PlainSessionKey::V6 { key: key.into() };

    if let Message::Encrypted { edata, .. } = &mut message {
        edata
            .decrypt(&key)
            .map_err(|_| MessageEncryptionError::DecryptWithKeyDecrypt)?;
        let mut output = vec![];
        edata
            .read_to_end(&mut output)
            .map_err(|_| MessageEncryptionError::DecryptWithKeyReadToEnd)?;
        Ok(output)
    } else {
        Err(MessageEncryptionError::DecryptWithKeyNotEncrypted)
    }
}
