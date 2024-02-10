use api_internal::{Configuration, InternalApi};
use config::{Config, InternalApiUrls};
use hyper::StatusCode;
use model::{
    AccessToken, Account, AccountIdInternal, AccountState, BooleanSetting, Capabilities, Profile,
    ProfileInternal,
};
use tracing::{error, info, warn};

use crate::{data::{read::ReadCommands, utils::AccessTokenManager}};
use crate::{
    app::{GetAccessTokens, GetConfig, GetInternalApi, ReadData, WriteData},
    data::WrappedWithInfo,
    result::{Result, WrappedContextExt, WrappedResultExt},
};

use super::InternalApiError;

pub enum AuthResponse {
    Ok,
    Unauthorized,
}

/// Check that API key is valid. Use this only from AccessToken checker handler.
/// This function will cache the account ID, so it can be found using normal
/// database calls after this runs.
pub async fn check_access_token<S: GetConfig + GetAccessTokens + GetInternalApi>(
    state: &S,
    key: AccessToken,
) -> Result<AuthResponse, InternalApiError> {
    if state
        .access_tokens()
        .access_token_exists(&key)
        .await
        .is_some()
    {
        Ok(AuthResponse::Ok)
    } else if !state.config().components().account {
        // Check AccessToken from external service

        let result = InternalApi::check_access_token(state.internal_api_client().account()?, key).await;

        match result {
            Ok(_res) => {
                // TODO: Cache this API key. Also needed for initializing
                // database tables.
                Ok(AuthResponse::Ok)
            }
            Err(api_internal::Error::ResponseError(response))
                if response.status.as_u16() == StatusCode::UNAUTHORIZED.as_u16() =>
            {
                // TODO: NOTE: Logging every error is not good as it would spam
                // the log, but maybe an error counter or logging just
                // once for a while.
                Ok(AuthResponse::Unauthorized)
            }
            Err(e) => Err(e).change_context(InternalApiError::ApiRequest),
        }
    } else {
        Ok(AuthResponse::Unauthorized)
    }
}
