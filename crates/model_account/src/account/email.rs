use diesel::{prelude::*, sql_types::SmallInt};
use model_server_data::EmailAddress;
use serde::{Deserialize, Serialize};
use simple_backend_model::SimpleDieselEnum;
use utoipa::ToSchema;

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
    pub email_verification_state_number: EmailSendingState,
    pub new_message_state_number: EmailSendingState,
    pub new_like_state_number: EmailSendingState,
    pub account_deletion_remainder_first_state_number: EmailSendingState,
    pub account_deletion_remainder_second_state_number: EmailSendingState,
    pub account_deletion_remainder_third_state_number: EmailSendingState,
    pub email_change_verification_state_number: EmailSendingState,
    pub email_change_cancellation_state_number: EmailSendingState,
}

impl AccountEmailSendingStateRaw {
    pub fn get_ref_to(&self, message: EmailMessages) -> &EmailSendingState {
        match message {
            EmailMessages::EmailVerification => &self.email_verification_state_number,
            EmailMessages::NewMessage => &self.new_message_state_number,
            EmailMessages::NewLike => &self.new_like_state_number,
            EmailMessages::AccountDeletionRemainderFirst => {
                &self.account_deletion_remainder_first_state_number
            }
            EmailMessages::AccountDeletionRemainderSecond => {
                &self.account_deletion_remainder_second_state_number
            }
            EmailMessages::AccountDeletionRemainderThird => {
                &self.account_deletion_remainder_third_state_number
            }
            EmailMessages::EmailChangeVerification => &self.email_change_verification_state_number,
            EmailMessages::EmailChangeCancellation => &self.email_change_cancellation_state_number,
        }
    }

    pub fn get_ref_mut_to(&mut self, message: EmailMessages) -> &mut EmailSendingState {
        match message {
            EmailMessages::EmailVerification => &mut self.email_verification_state_number,
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
            EmailMessages::EmailChangeVerification => {
                &mut self.email_change_verification_state_number
            }
            EmailMessages::EmailChangeCancellation => {
                &mut self.email_change_cancellation_state_number
            }
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct SendVerifyEmailMessageResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_already_verified: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_sending_failed: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_sending_timeout: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error_try_again_later_after_seconds: Option<u32>,
}

impl SendVerifyEmailMessageResult {
    pub fn ok() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn error_email_already_verified() -> Self {
        Self {
            error_email_already_verified: true,
            ..Default::default()
        }
    }

    pub fn error_email_sending_failed() -> Self {
        Self {
            error_email_sending_failed: true,
            ..Default::default()
        }
    }

    pub fn error_email_sending_timeout() -> Self {
        Self {
            error_email_sending_timeout: true,
            ..Default::default()
        }
    }

    pub fn error_try_again_later_after_seconds(seconds: u32) -> Self {
        Self {
            error_try_again_later_after_seconds: Some(seconds),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SetInitialEmail {
    pub email: EmailAddress,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct InitEmailChangeResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_sending_failed: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_sending_timeout: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error_try_again_later_after_seconds: Option<u32>,
}

impl InitEmailChangeResult {
    pub fn ok() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn error_email_sending_failed() -> Self {
        Self {
            error_email_sending_failed: true,
            ..Default::default()
        }
    }

    pub fn error_email_sending_timeout() -> Self {
        Self {
            error_email_sending_timeout: true,
            ..Default::default()
        }
    }

    pub fn error_try_again_later_after_seconds(seconds: u32) -> Self {
        Self {
            error_try_again_later_after_seconds: Some(seconds),
            ..Default::default()
        }
    }
}
