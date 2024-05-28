#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! This crate provides a wrapper for the internal API of the server.
//! Prevents exposing api_client crate model types to server code.

use api_client::apis::media_internal_api::{self};
pub use api_client::apis::{configuration::Configuration, Error};
use model::AccountId;

pub use crate::media_internal_api::InternalGetCheckModerationRequestForAccountError;

/// Wrapper for server internal API with correct model types.
pub struct InternalApi;

impl InternalApi {
    pub async fn media_check_moderation_request_for_account(
        configuration: &Configuration,
        account_id: AccountId,
    ) -> Result<(), Error<InternalGetCheckModerationRequestForAccountError>> {
        media_internal_api::internal_get_check_moderation_request_for_account(
            configuration,
            &account_id.to_string(),
        )
        .await
    }
}
