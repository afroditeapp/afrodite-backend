use axum::{Extension, extract::State};
use model_account::{AccountIdInternal, GetAssociationMembership, UpdateAssociationMembership};
use server_api::{
    S,
    app::{GetConfig, ReadData, WriteData},
    create_open_api_router, db_write,
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_GET_ASSOCIATION_MEMBERSHIP: &str = "/account_api/association_membership";

/// Get current association membership.
#[utoipa::path(
    get,
    path = PATH_GET_ASSOCIATION_MEMBERSHIP,
    responses(
        (status = 200, description = "Successful.", body = GetAssociationMembership),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_association_membership(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<GetAssociationMembership>, StatusCode> {
    ACCOUNT.get_association_membership.incr();

    let config = GetConfig::config(&state)
        .client_features_internal()
        .association
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if !config.user_can_view_existing_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let entry = state
        .read()
        .account()
        .association()
        .get_own_entry(account_id)
        .await?;

    Ok(GetAssociationMembership::from(entry).into())
}

const PATH_POST_ASSOCIATION_MEMBERSHIP: &str = "/account_api/association_membership";

/// Create or update association membership.
///
/// When membership already exists, the full name
/// and domicile fields are editable.
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

    let config = GetConfig::config(&state)
        .client_features_internal()
        .association
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let existing = state
        .read()
        .account()
        .association()
        .get_own_entry(account_id)
        .await?;

    if existing.is_some() {
        if !config.user_can_edit_existing_membership {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    } else {
        if !config.user_can_join_association {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let membership_type = config
        .membership_types
        .iter()
        .find(|t| t.id == data.membership_type)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if membership_type.admin_only {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // When editing an existing membership, only allow updating
    // full_name and domicile — keep the original membership_type.
    let data = if let Some(existing) = existing {
        UpdateAssociationMembership {
            membership_type: existing.membership_type,
            ..data
        }
    } else {
        data
    };

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

    let config = GetConfig::config(&state)
        .client_features_internal()
        .association
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if !config.user_can_edit_existing_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

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
