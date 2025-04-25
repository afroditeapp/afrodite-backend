use base64::Engine;
use diesel::prelude::Insertable;
use model::{AccountId, ChatMessageReport, MessageNumber, UnixTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateChatMessageReport {
    pub target: AccountId,
    pub backend_signed_message_base64: String,
    pub decryption_key_base64: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::chat_report_chat_message)]
#[diesel(check_for_backend(crate::Db))]
pub struct NewChatMessageReportInternal {
    pub message_sender_account_id_uuid: AccountId,
    pub message_receiver_account_id_uuid: AccountId,
    pub message_unix_time: UnixTime,
    pub message_number: MessageNumber,
    pub message_symmetric_key: Vec<u8>,
    pub client_message_bytes: Vec<u8>,
    pub backend_signed_message_bytes: Vec<u8>,
}

impl NewChatMessageReportInternal {
    pub fn to_chat_message_report(&self) -> ChatMessageReport {
        ChatMessageReport {
            sender: self.message_sender_account_id_uuid,
            receiver: self.message_receiver_account_id_uuid,
            message_time: self.message_unix_time,
            message_number: self.message_number,
            message_base64: base64::engine::general_purpose::STANDARD.encode(&self.client_message_bytes),
        }
    }
}
