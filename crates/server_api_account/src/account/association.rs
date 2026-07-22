use axum::{Extension, extract::State};
use model_account::{AccountIdInternal, AssociationMembership, UpdateAssociationMembership};
use server_api::{S, app::WriteData, create_open_api_router, db_write};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_ASSOCIATION_MEMBERSHIP: &str = "/account_api/association_membership";

/// Get current association membership.
#[utoipa::path(
    get,
    path = PATH_GET_ASSOCIATION_MEMBERSHIP,
    responses(
        (status = 200, description = "Successful.", body = Option<AssociationMembership>),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_association_membership(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<Option<AssociationMembership>>, StatusCode> {
    ACCOUNT.get_association_membership.incr();

    let entry = state
        .read()
        .account()
        .association()
        .get_own_entry(account_id)
        .await?;

    Ok(entry.into())
}

const PATH_POST_ASSOCIATION_MEMBERSHIP: &str = "/account_api/association_membership";

/// Create or update association membership.
#[utoipa::path(
    post,
    path = PATH_POST_ASSOCIATION_MEMBERSHIP,
    request_body = UpdateAssociationMembership,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_association_membership(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(data): Json<UpdateAssociationMembership>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_association_membership.incr();

    db_write!(state, move |cmds| {
        cmds.account()
            .association()
            .upsert_own_entry(account_id, data)
            .await
    })?;

    Ok(())
}

const PATH_DELETE_ASSOCIATION_MEMBERSHIP: &str = "/account_api/association_membership";

/// Remove association membership.
#[utoipa::path(
    delete,
    path = PATH_DELETE_ASSOCIATION_MEMBERSHIP,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_association_membership(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
    ACCOUNT.delete_association_membership.incr();

    db_write!(state, move |cmds| {
        cmds.account()
            .association()
            .remove_own_entry(account_id)
            .await
    })?;

    Ok(())
}

create_open_api_router!(
    fn router_association,
    get_association_membership,
    post_association_membership,
    delete_association_membership,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_ASSOCIATION_COUNTERS_LIST,
    get_association_membership,
    post_association_membership,
    delete_association_membership,
);
