use database::{DbReadMode, DieselDatabaseError};
use database_account::current::read::GetDbReadCommandsAccount;
use model::UnixTime;
use model_account::{
    AccountEmailSendingStateRaw, AccountStateTableRaw, AssociationMembershipDataExportEntry,
    EmailAddressState, EmailChangeLimits, EmailLoginLimits, EmailLoginTokens,
};
use model_chat::AccountAppNotificationSettings;
use serde::Serialize;
use server_data::data_export::SourceAccount;

// TODO(future): Add news to data export. This is low priority task as
//               only admins can create or edit news.

#[derive(Serialize)]
pub struct UserDataExportJsonAccount {
    email_address_state: EmailAddressState,
    email_sending_states: AccountEmailSendingStateRaw,
    account_state_table: AccountStateTableRaw,
    account_notification_settings: AccountAppNotificationSettings,
    email_login_tokens: EmailLoginTokens,
    email_login_token_time: Option<UnixTime>,
    email_login_limits: Option<EmailLoginLimits>,
    email_change_limits: Option<EmailChangeLimits>,
    email_verification_token: Option<Vec<u8>>,
    email_verification_token_time: Option<UnixTime>,
    association_membership: Option<AssociationMembershipDataExportEntry>,
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
            email_sending_states: current.account().email().email_sending_states(id)?,
            account_state_table: current.account().data().account_state_table_raw(id)?,
            account_notification_settings: current
                .account()
                .notification()
                .app_notification_settings(id)?,
            email_login_tokens: current.account().email().email_login_tokens(id)?,
            email_login_token_time: current.account().email().email_login_token_time(id)?,
            email_login_limits: current.account().email().email_login_limits(id)?,
            email_change_limits: current.account().email().email_change_limits(id)?,
            email_verification_token,
            email_verification_token_time,
            association_membership: current.account().association().get_own_entry(id)?.map(|e| {
                AssociationMembershipDataExportEntry {
                    creation_unix_time: e.creation_unix_time,
                    edit_unix_time: e.edit_unix_time,
                    full_name: e.full_name,
                    domicile: e.domicile,
                    membership_type: e.membership_type,
                }
            }),
            note: "If you created or edited news, that data is not currently included here.",
        };
        Ok(data)
    }
}
