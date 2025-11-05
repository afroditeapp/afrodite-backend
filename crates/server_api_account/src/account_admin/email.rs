use axum::{
    Extension,
    extract::{Path, State},
};
use model::{AccountId, Permissions};
use model_account::{EmailAddressStateForAdmin, InitEmailChangeAdmin, InitEmailChangeResult};
use server_api::{S, create_open_api_router, db_write};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::{
    account::email::init_email_change_impl,
    app::{GetAccounts, ReadData, WriteData},
    utils::{Json, StatusCode},
};

pub const PATH_GET_EMAIL_ADDRESS_STATE: &str = "/account_api/get_email_address_state/{aid}";

/// Get email address state for admin.
///
/// Requires `admin_view_email_address` permission.
#[utoipa::path(
    get,
    path = PATH_GET_EMAIL_ADDRESS_STATE,
    params(AccountId),
    responses(
        (status = 200, description = "Successfull.", body = EmailAddressStateForAdmin),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_email_address_state(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Path(target_account): Path<AccountId>,
) -> Result<Json<EmailAddressStateForAdmin>, StatusCode> {
    ACCOUNT_ADMIN.get_email_address_state.incr();

    if !api_caller_permissions.admin_view_email_address {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let target_account_internal = state.get_internal_id(target_account).await?;

    let data = state
        .read()
        .account()
        .account_data(target_account_internal)
        .await?;

    let email_state = EmailAddressStateForAdmin {
        email: data.email,
        email_change: data.email_change,
        email_change_verified: data.email_change_verified,
        email_login_enabled: data.email_login_enabled,
    };

    Ok(Json(email_state))
}

pub const PATH_POST_ADMIN_CANCEL_EMAIL_CHANGE: &str =
    "/account_api/admin_cancel_email_change/{aid}";

/// Cancel email changing process for any account.
///
/// # Access
///
/// Permission [model::Permissions::admin_change_email_address] is required.
#[utoipa::path(
    post,
    path = PATH_POST_ADMIN_CANCEL_EMAIL_CHANGE,
    params(AccountId),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_admin_cancel_email_change(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Path(target_account): Path<AccountId>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.post_admin_cancel_email_change.incr();

    if !api_caller_permissions.admin_change_email_address {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let target_account_internal = state.get_internal_id(target_account).await?;

    db_write!(state, move |cmds| {
        cmds.account()
            .email()
            .cancel_email_change(target_account_internal)
            .await
    })?;

    Ok(())
}

pub const PATH_POST_ADMIN_INIT_EMAIL_CHANGE: &str = "/account_api/admin_init_email_change";

/// Initiate email change process for any account by providing a new email address.
///
/// This is the admin version of the email change endpoint.
///
/// # Access
///
/// Permission [model::Permissions::admin_change_email_address] is required.
#[utoipa::path(
    post,
    path = PATH_POST_ADMIN_INIT_EMAIL_CHANGE,
    request_body = InitEmailChangeAdmin,
    responses(
        (status = 200, description = "Successfull.", body = InitEmailChangeResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_admin_init_email_change(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(request): Json<InitEmailChangeAdmin>,
) -> Result<Json<InitEmailChangeResult>, StatusCode> {
    ACCOUNT_ADMIN.post_admin_init_email_change.incr();

    if !api_caller_permissions.admin_change_email_address {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let target_account_internal = state.get_internal_id(request.account_id).await?;

    let result = init_email_change_impl(&state, target_account_internal, request.new_email).await?;
    Ok(result.into())
}

create_open_api_router!(
    fn router_admin_email,
    get_email_address_state,
    post_admin_cancel_email_change,
    post_admin_init_email_change,
);

create_counters!(
    AccountAdminCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_EMAIL_COUNTERS_LIST,
    get_email_address_state,
    post_admin_cancel_email_change,
    post_admin_init_email_change,
);
