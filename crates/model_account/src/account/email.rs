use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use simple_backend_model::SimpleDieselEnum;

use crate::{EmailMessages, EnumParsingError, schema_sqlite_types::Integer};

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
)]
#[diesel(sql_type = Integer)]
#[repr(i64)]
pub enum EmailSendingState {
    /// Backend has not yet tried to send the email.
    NotSent = 0,
    /// Backend moved the email to send queue.
    SendRequested = 1,
    /// SMTP server returned a positive response.
    SentSuccessfully = 2,
}

impl TryFrom<i64> for EmailSendingState {
    type Error = EnumParsingError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::NotSent,
            1 => Self::SendRequested,
            2 => Self::SentSuccessfully,
            _ => return Err(EnumParsingError::ParsingError(value)),
        };

        Ok(value)
    }
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
}

impl AccountEmailSendingStateRaw {
    pub fn get_ref_mut_to(&mut self, message: EmailMessages) -> &mut EmailSendingState {
        match message {
            EmailMessages::AccountRegistered => &mut self.account_registered_state_number,
            EmailMessages::NewMessage => &mut self.new_message_state_number,
            EmailMessages::NewLike => &mut self.new_like_state_number,
        }
    }
}
