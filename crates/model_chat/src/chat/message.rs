use model::{AccountId, MessageNumber, PublicKeyId, UnixTime};
use simple_backend_utils::UuidBase64Url;

pub struct SignedMessageData {
    /// Sender of the message.
    pub sender: AccountId,
    /// Receiver of the message.
    pub receiver: AccountId,
    pub sender_public_key_id: PublicKeyId,
    pub receiver_public_key_id: PublicKeyId,
    pub mn: MessageNumber,
    /// Unix time when server received the message.
    pub unix_time: UnixTime,
    pub message: Vec<u8>,
}

impl SignedMessageData {
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        Self::parse_internal(data).ok_or("Parsing failure: not enough data".to_string())?
    }

    fn parse_internal(data: &[u8]) -> Option<Result<Self, String>> {
        let mut d = data.iter().copied();

        let version = d.next()?;
        if version != 1 {
            return Some(Err(format!("Data version {version}, expected: 1")));
        }
        let sender = parse_account_id(&mut d)?;
        let receiver = parse_account_id(&mut d)?;
        let sender_public_key_id = parse_minimal_i64(&mut d)?;
        let receiver_public_key_id = parse_minimal_i64(&mut d)?;
        let mn = parse_minimal_i64(&mut d)?;
        let ut = parse_minimal_i64(&mut d)?;
        let message = d.collect();

        Some(Ok(SignedMessageData {
            sender,
            receiver,
            sender_public_key_id: PublicKeyId {
                id: sender_public_key_id,
            },
            receiver_public_key_id: PublicKeyId {
                id: receiver_public_key_id,
            },
            mn: MessageNumber { mn },
            unix_time: UnixTime { ut },
            message,
        }))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        // Version
        bytes.push(1);
        // Sender UUID big-endian bytes (16 bytes)
        bytes.extend_from_slice(self.sender.aid.as_bytes());
        // Receiver UUID big-endian bytes (16 bytes)
        bytes.extend_from_slice(self.receiver.aid.as_bytes());
        add_minimal_i64(&mut bytes, self.sender_public_key_id.id);
        add_minimal_i64(&mut bytes, self.receiver_public_key_id.id);
        add_minimal_i64(&mut bytes, self.mn.mn);
        add_minimal_i64(&mut bytes, self.unix_time.ut);
        // Sent message data
        bytes.extend_from_slice(&self.message);
        bytes
    }
}

pub fn add_minimal_i64(bytes: &mut Vec<u8>, value: i64) {
    if let Ok(v) = TryInto::<i8>::try_into(value) {
        bytes.push(1);
        bytes.extend_from_slice(&v.to_le_bytes());
    } else if let Ok(v) = TryInto::<i16>::try_into(value) {
        bytes.push(2);
        bytes.extend_from_slice(&v.to_le_bytes());
    } else if let Ok(v) = TryInto::<i32>::try_into(value) {
        bytes.push(4);
        bytes.extend_from_slice(&v.to_le_bytes());
    } else {
        bytes.push(8);
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn parse_account_id(d: &mut impl Iterator<Item = u8>) -> Option<AccountId> {
    let bytes: Vec<u8> = d.by_ref().take(16).collect();
    let bytes = TryInto::<[u8; 16]>::try_into(bytes).ok()?;
    Some(AccountId::new_base_64_url(UuidBase64Url::from_bytes(bytes)))
}

fn parse_minimal_i64(d: &mut impl Iterator<Item = u8>) -> Option<i64> {
    let count = d.next()?;
    let number: i64 = if count == 1 {
        i8::from_le_bytes([d.next()?]).into()
    } else if count == 2 {
        i16::from_le_bytes([d.next()?, d.next()?]).into()
    } else if count == 4 {
        i32::from_le_bytes([d.next()?, d.next()?, d.next()?, d.next()?]).into()
    } else if count == 8 {
        i64::from_le_bytes([
            d.next()?,
            d.next()?,
            d.next()?,
            d.next()?,
            d.next()?,
            d.next()?,
            d.next()?,
            d.next()?,
        ])
    } else {
        return None;
    };

    Some(number)
}
