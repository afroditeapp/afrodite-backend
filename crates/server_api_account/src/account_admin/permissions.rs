use axum::{
    Extension,
    extract::{Path, State},
};
use model::{AccountId, EventToClientInternal, Permissions};
use model_account::GetAllAdminsResult;
use server_api::{
    S,
    app::{GetAccounts, ReadData, WriteData},
    create_open_api_router, db_write,
};
use server_data::read::GetReadCommandsCommon;
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_GET_PERMISSIONS: &str = "/account_api/get_permissions/{aid}";

/// Get [model::Permissions] for specific account.
///
/// # Access
///
/// Permission [model::Permissions::admin_view_permissions] is required.
#[utoipa::path(
    get,
    path = PATH_GET_PERMISSIONS,
    params(AccountId),
    responses(
        (status = 200, description = "Successfull.", body = Permissions),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_permissions(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Path(account_id): Path<AccountId>,
) -> Result<Json<Permissions>, StatusCode> {
    ACCOUNT_ADMIN.get_permissions.incr();

    if !permissions.admin_view_permissions {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account_id).await?;

    let permissions = state.read().common().account(internal_id).await?;

    Ok(permissions.permissions().into())
}

const PATH_GET_ALL_ADMINS: &str = "/account_api/get_all_admins";

/// Get all admins
///
/// # Access
///
/// Permission [model_account::Permissions::admin_view_permissions] is required.
#[utoipa::path(
    get,
    path = PATH_GET_ALL_ADMINS,
    responses(
        (status = 200, description = "Successfull.", body = GetAllAdminsResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_all_admins(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetAllAdminsResult>, StatusCode> {
    ACCOUNT_ADMIN.get_all_admins.incr();

    if !permissions.admin_view_permissions {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let admins = state
        .read()
        .account_admin()
        .permissions()
        .all_admins()
        .await?;

    Ok(admins.into())
}

const PATH_POST_SET_PERMISSIONS: &str = "/account_api/set_permissions/{aid}";

/// Set permissions for account
///
/// # Access
///
/// Permission [model_account::Permissions::admin_edit_permissions] is required.
#[utoipa::path(
    post,
    path = PATH_POST_SET_PERMISSIONS,
    params(AccountId),
    request_body = Permissions,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_set_permissions(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Path(account): Path<AccountId>,
    Json(new_permissions): Json<Permissions>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.post_set_permissions.incr();

    if !api_caller_permissions.admin_edit_permissions {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(account).await?;

    db_write!(state, move |cmds| {
        cmds.account_admin()
            .permissions()
            .set_permissions(internal_id, new_permissions)
            .await?;

        cmds.events()
            .send_connected_event(internal_id.uuid, EventToClientInternal::AccountStateChanged)
            .await?;

        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(fn router_admin_permissions, get_permissions, get_all_admins, post_set_permissions,);

create_counters!(
    AccountCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_PERMISSIONS_COUNTERS_LIST,
    get_permissions,
    get_all_admins,
    post_set_permissions,
);
