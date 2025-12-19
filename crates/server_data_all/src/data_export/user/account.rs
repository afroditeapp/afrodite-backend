use database::{DbReadMode, DieselDatabaseError};
use database_account::current::read::GetDbReadCommandsAccount;
use model::UnixTime;
use model_account::{
    AccountEmailSendingStateRaw, AccountSetup, AccountStateTableRaw, EmailAddressState,
    EmailLoginTokens,
};
use model_chat::AccountAppNotificationSettings;
use serde::Serialize;
use server_data::data_export::SourceAccount;

// TODO(future): Add news to data export. This is low priority task as
//               only admins can create or edit news.

#[derive(Serialize)]
pub struct UserDataExportJsonAccount {
    email_address_state: EmailAddressState,
    account_setup: AccountSetup,
    email_sending_states: AccountEmailSendingStateRaw,
    account_state_table: AccountStateTableRaw,
    account_notification_settings: AccountAppNotificationSettings,
    email_login_tokens: EmailLoginTokens,
    email_login_token_time: Option<UnixTime>,
    email_verification_token: Option<Vec<u8>>,
    email_verification_token_time: Option<UnixTime>,
    note: &'static str,
}

impl UserDataExportJsonAccount {
    pub fn query(
        current: &mut DbReadMode,
        id: SourceAccount,
    ) -> error_stack::Result<Self, DieselDatabaseError> {
        let id = id.0;
        let (email_verification_token, email_verification_token_time) =
            current.account().email().email_verification_token(id)?;
        let data = Self {
            email_address_state: current.account().data().email_address_state(id)?,
            account_setup: current.account().data().account_setup(id)?,
            email_sending_states: current.account().email().email_sending_states(id)?,
            account_state_table: current.account().data().account_state_table_raw(id)?,
            account_notification_settings: current
                .account()
                .notification()
                .app_notification_settings(id)?,
            email_login_tokens: current.account().email().email_login_tokens(id)?,
            email_login_token_time: current.account().email().email_login_token_time(id)?,
            email_verification_token,
            email_verification_token_time,
            note: "If you created or edited news, that data is not currently included here.",
        };
        Ok(data)
    }
}
