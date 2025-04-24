use model::{AccountId, MessageNumber, UnixTime};

pub struct SignedMessageData {
    /// Sender of the message.
    pub sender: AccountId,
    /// Receiver of the message.
    pub receiver: AccountId,
    pub mn: MessageNumber,
    /// Unix time when server received the message.
    pub unix_time: UnixTime,
    pub message: Vec<u8>,
}

impl SignedMessageData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        // Version
        bytes.push(1);
        // Sender UUID big-endian bytes (16 bytes)
        bytes.extend_from_slice(self.sender.aid.as_bytes());
        // Receiver UUID big-endian bytes (16 bytes)
        bytes.extend_from_slice(self.receiver.aid.as_bytes());
        // Variable lenght i64 value (u8 byte count and litle-endian data)
        add_minimal_i64(&mut bytes, self.mn.mn);
        // Variable lenght i64 value (u8 byte count and litle-endian data)
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
