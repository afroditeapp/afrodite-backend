use database::{DbReadMode, DieselDatabaseError};
use database_chat::current::read::GetDbReadCommandsChat;
use model::{
    AccountInteractionInternal, AdminDataExportPendingMessage, DataExportLatestSeenMessage,
};
use model_chat::DataExportMessageDeliveryInfo;
use serde::Serialize;
use server_data::data_export::SourceAccount;

#[derive(Serialize)]
pub struct AdminDataExportJsonChat {
    account_interactions: Vec<AccountInteractionInternal>,
    pending_messages: Vec<AdminDataExportPendingMessage>,
    message_delivery_info: Vec<DataExportMessageDeliveryInfo>,
    latest_seen_message_viewer: Vec<DataExportLatestSeenMessage>,
    latest_seen_message_sender: Vec<DataExportLatestSeenMessage>,
}

impl AdminDataExportJsonChat {
    pub fn query(
        current: &mut DbReadMode,
        id: SourceAccount,
    ) -> error_stack::Result<Self, DieselDatabaseError> {
        let id = id.0;
        let data = Self {
            account_interactions: current
                .chat()
                .interaction()
                .all_related_account_interactions(id)?,
            pending_messages: current
                .chat()
                .message()
                .admin_data_export_pending_messages(id)?,
            message_delivery_info: current
                .chat()
                .message()
                .data_export_delivery_info_from_me(id)?,
            latest_seen_message_viewer: current
                .chat()
                .message()
                .data_export_latest_sent_message_numbers_viewer(id)?,
            latest_seen_message_sender: current
                .chat()
                .message()
                .data_export_latest_sent_message_numbers_sender(id)?,
        };
        Ok(data)
    }
}
