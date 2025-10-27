use diesel::{prelude::*, sql_types::SmallInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::SimpleDieselEnum;

use crate::EmailMessages;

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
    num_enum::TryFromPrimitive,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum EmailSendingState {
    /// Backend has not yet tried to send the email.
    NotSent = 0,
    /// Backend moved the email to send queue.
    SendRequested = 1,
    /// SMTP server returned a positive response.
    SentSuccessfully = 2,
}

impl Default for EmailSendingState {
    fn default() -> Self {
        Self::NotSent
    }
}

#[derive(Debug, Clone, Default, Serialize, Queryable, Selectable, AsChangeset, Insertable)]
#[diesel(table_name = crate::schema::account_email_sending_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountEmailSendingStateRaw {
    pub account_registered_state_number: EmailSendingState,
    pub new_message_state_number: EmailSendingState,
    pub new_like_state_number: EmailSendingState,
    pub account_deletion_remainder_first_state_number: EmailSendingState,
    pub account_deletion_remainder_second_state_number: EmailSendingState,
    pub account_deletion_remainder_third_state_number: EmailSendingState,
}

impl AccountEmailSendingStateRaw {
    pub fn get_ref_mut_to(&mut self, message: EmailMessages) -> &mut EmailSendingState {
        match message {
            EmailMessages::AccountRegistered => &mut self.account_registered_state_number,
            EmailMessages::NewMessage => &mut self.new_message_state_number,
            EmailMessages::NewLike => &mut self.new_like_state_number,
            EmailMessages::AccountDeletionRemainderFirst => {
                &mut self.account_deletion_remainder_first_state_number
            }
            EmailMessages::AccountDeletionRemainderSecond => {
                &mut self.account_deletion_remainder_second_state_number
            }
            EmailMessages::AccountDeletionRemainderThird => {
                &mut self.account_deletion_remainder_third_state_number
            }
        }
    }
}
