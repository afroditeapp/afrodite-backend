use axum::{
    Extension,
    extract::{Query, State},
};
use model_account::{
    AccountIdInternal, CustomEmailId, GetCustomEmailConfig, GetCustomEmailListParams, Permissions,
    SendCustomEmail, UpdateCustomEmail,
};
use server_api::{S, app::GetConfig, create_open_api_router, db_write};
use server_data::read::GetReadCommandsCommon;
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::{ReadData, WriteData};

const PATH_GET_CUSTOM_EMAIL_CONFIG: &str = "/account_api/custom_email_config";

#[utoipa::path(
    get,
    path = PATH_GET_CUSTOM_EMAIL_CONFIG,
    responses(
        (status = 200, description = "Success.", body = GetCustomEmailConfig),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_custom_email_config(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetCustomEmailConfig>, StatusCode> {
    ACCOUNT.get_custom_email_config.incr();

    if !permissions.admin_custom_email {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(GetCustomEmailConfig {
        email_body_is_html: state
            .config()
            .email_content()
            .email_body_content_type_is_html(),
    }))
}

const PATH_GET_CUSTOM_EMAIL_LIST: &str = "/account_api/custom_email_list";

/// List all custom emails, newest first.
#[utoipa::path(
    get,
    path = PATH_GET_CUSTOM_EMAIL_LIST,
    params(GetCustomEmailListParams),
    responses(
        (status = 200, description = "Success.", body = Vec<model_account::CustomEmail>),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_custom_email_list(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetCustomEmailListParams>,
) -> Result<Json<Vec<model_account::CustomEmail>>, StatusCode> {
    ACCOUNT.get_custom_email_list.incr();

    if !permissions.admin_custom_email {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let emails = state
        .read()
        .account_admin()
        .custom_email()
        .custom_email_list_page(params.page)
        .await?;
    Ok(emails.into())
}

const PATH_POST_CREATE_CUSTOM_EMAIL: &str = "/account_api/create_custom_email";

/// Create a new custom email message draft.
#[utoipa::path(
    post,
    path = PATH_POST_CREATE_CUSTOM_EMAIL,
    responses(
        (status = 200, description = "Success.", body = CustomEmailId),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_create_custom_email(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<CustomEmailId>, StatusCode> {
    ACCOUNT.post_create_custom_email.incr();

    if !permissions.admin_custom_email {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let email_id = db_write!(state, move |cmds| cmds
        .account_admin()
        .custom_email()
        .create_custom_email(account_id)
        .await)?;
    Ok(email_id.into())
}

const PATH_POST_UPDATE_CUSTOM_EMAIL: &str = "/account_api/update_custom_email";

/// Update a custom email message draft.
///
/// Translation with "default" locale must exist.
#[utoipa::path(
    post,
    path = PATH_POST_UPDATE_CUSTOM_EMAIL,
    request_body(content = UpdateCustomEmail),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_update_custom_email(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(data): Json<UpdateCustomEmail>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_update_custom_email.incr();

    if !permissions.admin_custom_email {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Validate that "default" locale translation exists
    if !data.translations.iter().any(|t| t.locale == "default") {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write!(state, move |cmds| {
        cmds.account_admin()
            .custom_email()
            .update_custom_email(data)
            .await
    })?;

    Ok(())
}

const PATH_POST_SEND_CUSTOM_EMAIL_TO_ALL_ACCOUNTS: &str =
    "/account_api/send_custom_email_to_all_accounts";

#[utoipa::path(
    post,
    path = PATH_POST_SEND_CUSTOM_EMAIL_TO_ALL_ACCOUNTS,
    request_body(content = SendCustomEmail),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_send_custom_email_to_all_accounts(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(data): Json<SendCustomEmail>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_send_custom_email_to_all_accounts.incr();

    if !permissions.admin_custom_email {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let account_ids = state.read().common().account_ids_internal_vec().await?;

    db_write!(state, move |cmds| {
        cmds.account_admin()
            .custom_email()
            .send_custom_email(data.email_id, account_ids)
            .await
    })?;

    Ok(())
}

const PATH_POST_SEND_CUSTOM_EMAIL_DRAFT_TO_MY_EMAIL_ADDRESS: &str =
    "/account_api/send_custom_email_draft_to_my_email_address";

#[utoipa::path(
    post,
    path = PATH_POST_SEND_CUSTOM_EMAIL_DRAFT_TO_MY_EMAIL_ADDRESS,
    request_body(content = SendCustomEmail),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_send_custom_email_draft_to_my_email_address(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Json(data): Json<SendCustomEmail>,
) -> Result<(), StatusCode> {
    ACCOUNT
        .post_send_custom_email_draft_to_my_email_address
        .incr();

    if !permissions.admin_custom_email {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write!(state, move |cmds| {
        cmds.account_admin()
            .custom_email()
            .send_custom_email_draft_to_target(data.email_id, account_id)
            .await
    })?;

    Ok(())
}

create_open_api_router!(
    fn router_admin_custom_email,
    get_custom_email_config,
    get_custom_email_list,
    post_create_custom_email,
    post_update_custom_email,
    post_send_custom_email_to_all_accounts,
    post_send_custom_email_draft_to_my_email_address,
);

create_counters!(
    AccountAdminCounterCustomEmail,
    ACCOUNT,
    ACCOUNT_ADMIN_CUSTOM_EMAIL_COUNTERS_LIST,
    get_custom_email_config,
    get_custom_email_list,
    post_create_custom_email,
    post_update_custom_email,
    post_send_custom_email_to_all_accounts,
    post_send_custom_email_draft_to_my_email_address,
);
