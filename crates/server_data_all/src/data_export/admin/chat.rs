use database::{DbReadMode, DieselDatabaseError};
use database_chat::current::read::GetDbReadCommandsChat;
use model::{AccountInteractionInternal, AdminDataExportPendingMessage};
use serde::Serialize;
use server_data::data_export::SourceAccount;

#[derive(Serialize)]
pub struct AdminDataExportJsonChat {
    account_interactions: Vec<AccountInteractionInternal>,
    pending_messages: Vec<AdminDataExportPendingMessage>,
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
        };
        Ok(data)
    }
}
