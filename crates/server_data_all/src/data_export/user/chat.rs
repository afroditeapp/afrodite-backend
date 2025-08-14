use database::{DbReadMode, DieselDatabaseError};
use database_chat::current::read::GetDbReadCommandsChat;
use model::DataExportPendingMessage;
use model_chat::{
    ChatAppNotificationSettings, ChatStateRaw, DailyLikesLeftInternal, DataExportPublicKey,
};
use serde::Serialize;
use server_data::data_export::SourceAccount;

#[derive(Serialize)]
pub struct UserDataExportJsonChat {
    chat_state: ChatStateRaw,
    max_public_key_count_account_config: i64,
    public_keys: Vec<DataExportPublicKey>,
    daily_likes: DailyLikesLeftInternal,
    pending_messages: Vec<DataExportPendingMessage>,
    chat_app_notification_settings: ChatAppNotificationSettings,
    note: &'static str,
}

impl UserDataExportJsonChat {
    pub fn query(
        current: &mut DbReadMode,
        id: SourceAccount,
    ) -> error_stack::Result<Self, DieselDatabaseError> {
        let id = id.0;
        let data = Self {
            chat_state: current.chat().chat_state(id)?,
            max_public_key_count_account_config: current
                .chat()
                .public_key()
                .max_public_key_count_account_config(id)?,
            public_keys: current.chat().public_key().all_public_keys(id)?,
            daily_likes: current.chat().limits().daily_likes_left(id)?,
            pending_messages: current.chat().message().data_export_pending_messages(id)?,
            chat_app_notification_settings: current
                .chat()
                .notification()
                .app_notification_settings(id)?,
            note: "Info about interactions with other accounts is not included.",
        };
        Ok(data)
    }
}
