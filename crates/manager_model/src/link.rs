use std::num::Wrapping;

#[derive(Debug, Clone, Copy, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum JsonRpcLinkMessageType {
    Empty = 0,
    ServerRequest = 1,
    ServerResponse = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonRpcLinkMessage {
    pub header: JsonRpcLinkHeader,
    pub data: String,
}

impl JsonRpcLinkMessage {
    pub fn empty() -> Self {
        Self {
            header: JsonRpcLinkHeader {
                message_type: JsonRpcLinkMessageType::Empty,
                sequence_number: Wrapping(0),
            },
            data: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonRpcLinkHeader {
    pub message_type: JsonRpcLinkMessageType,
    pub sequence_number: Wrapping<u32>,
}
