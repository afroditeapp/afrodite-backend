
use model::{AccountIdInternal, SetProfileSetup};
use server_data::result::WrappedContextExt;

use super::InternalApiError;
use crate::{
    app::GetInternalApi,
    result::Result,
};

pub async fn set_profile_setup_using_internal_api_call<S: GetInternalApi>(
    _state: &S,
    _id: AccountIdInternal,
    _data: SetProfileSetup,
) -> Result<(), InternalApiError> {
    // TODO(microservice): implement setting SetProfileSetup to
    // profile server
    Err(InternalApiError::ApiRequest.report())
}
