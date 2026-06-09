use tracing::warn;

pub fn message_bytes_to_text(bytes: &[u8]) -> Option<String> {
    let mut current: &[u8] = bytes;
    loop {
        match message_bytes_to_text_internal(current) {
            None => return None,
            Some(ParsedMessage::Text(text)) => return Some(text),
            Some(ParsedMessage::MessageBytes(inner)) => current = inner,
        }
    }
}

enum ParsedMessage<'a> {
    Text(String),
    MessageBytes(&'a [u8]),
}

fn message_bytes_to_text_internal<'a>(bytes: &'a [u8]) -> Option<ParsedMessage<'a>> {
    if bytes.is_empty() {
        warn!("Empty message bytes — cannot decode");
        return None;
    }

    let packet_type = bytes[0];

    match packet_type {
        // Text message (0): 2 bytes LE u16 length, then UTF-8 data
        // Message with reference (2): 2 bytes LE u16 text length, UTF-8 text, then UTF-8 message ID
        0 | 2 => {
            let message_type_str = if packet_type == 0 {
                "Text message"
            } else {
                "MessageWithReference"
            };
            if bytes.len() < 3 {
                warn!("{message_type_str} packet too short (len={})", bytes.len());
                return None;
            }
            let utf8_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
            if bytes.len() < 3 + utf8_len {
                warn!(
                    "{message_type_str} truncated: header says {utf8_len} bytes but packet has {}",
                    bytes.len().saturating_sub(3)
                );
                return None;
            }
            let text_bytes = &bytes[3..3 + utf8_len];
            match std::str::from_utf8(text_bytes) {
                Ok(text) => Some(ParsedMessage::Text(text.to_string())),
                Err(e) => {
                    warn!("{message_type_str} invalid UTF-8: {e}");
                    None
                }
            }
        }

        // Video call invitation: no additional data
        1 => Some(ParsedMessage::Text("Video call invitation".to_string())),

        // Resent message: 8 bytes message number (i64 LE), 8 bytes sent unix time (i64 LE),
        // 1 byte message ID length, message ID bytes, then original message bytes
        3 => {
            if bytes.len() < 18 {
                warn!("ResentMessage packet too short (len={})", bytes.len());
                return None;
            }
            let message_id_len = bytes[17] as usize;
            let header_end = 18 + message_id_len;
            if bytes.len() < header_end {
                warn!(
                    "ResentMessage truncated: message ID length {message_id_len} but packet has {}",
                    bytes.len().saturating_sub(18)
                );
                return None;
            }
            let inner_bytes = &bytes[header_end..];
            Some(ParsedMessage::MessageBytes(inner_bytes))
        }

        // Unknown packet type
        other => {
            warn!("Unknown message packet type: {other}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bytes() {
        assert!(message_bytes_to_text(&[]).is_none());
    }

    #[test]
    fn unknown_packet_type() {
        let bytes = vec![0xFF];
        assert!(message_bytes_to_text(&bytes).is_none());
    }

    #[test]
    fn text_message() {
        let text = "Hello, world!";
        let text_bytes = text.as_bytes();
        let len = (text_bytes.len() as u16).to_le_bytes();
        let mut packet = vec![0u8, len[0], len[1]];
        packet.extend_from_slice(text_bytes);
        assert_eq!(
            message_bytes_to_text(&packet).as_deref(),
            Some("Hello, world!")
        );
    }

    #[test]
    fn text_message_truncated() {
        // Header says 100 bytes but only 5 provided
        let len = 100u16.to_le_bytes();
        let packet = vec![0u8, len[0], len[1], b'h', b'e', b'l', b'l', b'o'];
        assert!(message_bytes_to_text(&packet).is_none());
    }

    #[test]
    fn text_message_too_short() {
        let packet = vec![0u8];
        assert!(message_bytes_to_text(&packet).is_none());
    }

    #[test]
    fn video_call_invitation() {
        let packet = vec![1u8];
        assert_eq!(
            message_bytes_to_text(&packet).as_deref(),
            Some("Video call invitation")
        );
    }

    #[test]
    fn message_with_reference() {
        let text = "referenced text";
        let ref_id = "msg-123";
        let text_bytes = text.as_bytes();
        let id_bytes = ref_id.as_bytes();
        let len = (text_bytes.len() as u16).to_le_bytes();
        let mut packet = vec![2u8, len[0], len[1]];
        packet.extend_from_slice(text_bytes);
        packet.extend_from_slice(id_bytes);
        assert_eq!(
            message_bytes_to_text(&packet).as_deref(),
            Some("referenced text")
        );
    }

    #[test]
    fn message_with_reference_truncated() {
        let len = 100u16.to_le_bytes();
        // Only 5 text bytes provided instead of 100
        let packet = vec![2u8, len[0], len[1], b'h', b'e', b'l', b'l', b'o'];
        assert!(message_bytes_to_text(&packet).is_none());
    }

    #[test]
    fn resent_message_recursive_text() {
        // Build inner text message: "inner"
        let inner_text = "inner";
        let inner_bytes = inner_text.as_bytes();
        let inner_len = (inner_bytes.len() as u16).to_le_bytes();
        let mut inner_packet = vec![0u8, inner_len[0], inner_len[1]];
        inner_packet.extend_from_slice(inner_bytes);

        // Wrap in resent: 8 bytes msg num, 8 bytes time, 1 byte id len, id, inner
        let msg_num = 42i64.to_le_bytes();
        let sent_time = 1234567890i64.to_le_bytes();
        let msg_id = "rid-1";
        let msg_id_bytes = msg_id.as_bytes();
        let mut packet = vec![3u8];
        packet.extend_from_slice(&msg_num);
        packet.extend_from_slice(&sent_time);
        packet.push(msg_id_bytes.len() as u8);
        packet.extend_from_slice(msg_id_bytes);
        packet.extend_from_slice(&inner_packet);

        assert_eq!(message_bytes_to_text(&packet).as_deref(), Some("inner"));
    }

    #[test]
    fn resent_message_truncated() {
        let packet = vec![3u8, 0, 0, 0, 0, 0, 0, 0, 0]; // only 9 bytes
        assert!(message_bytes_to_text(&packet).is_none());
    }

    #[test]
    fn non_utf8_text_message() {
        // Invalid UTF-8 bytes: 0xFF is not valid UTF-8
        let len = 1u16.to_le_bytes();
        let packet = vec![0u8, len[0], len[1], 0xFF];
        assert!(message_bytes_to_text(&packet).is_none());
    }
}
