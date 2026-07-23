use axum::{Extension, extract::State};
use model::{AccountId, Permissions};
use model_account::{
    AssociationMember, AssociationMembersPage, GetAssociationMembersPage,
    ManualAssociationMembershipRegistry, ManualAssociationMembershipRegistryInput,
    UpdateAssociationMembershipType,
};
use server_api::{
    S,
    app::{GetAccounts, GetConfig, ReadData},
    create_open_api_router, db_write,
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::{
    app::WriteData,
    utils::{Json, StatusCode},
};

const PATH_GET_MANUAL_ASSOCIATION_MEMBERSHIP_REGISTRY: &str =
    "/account_api/manual_association_membership_registry";

/// Get the manual association membership registry.
///
/// # Access
///
/// Permission [model::Permissions::admin_view_association_membership] is required.
#[utoipa::path(
    get,
    path = PATH_GET_MANUAL_ASSOCIATION_MEMBERSHIP_REGISTRY,
    responses(
        (status = 200, description = "Successful.", body = ManualAssociationMembershipRegistry),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_manual_association_membership_registry(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<ManualAssociationMembershipRegistry>, StatusCode> {
    ACCOUNT_ADMIN
        .get_manual_association_membership_registry
        .incr();

    if !permissions.admin_view_association_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let registry = state
        .read()
        .account_admin()
        .association()
        .get_manual_registry()
        .await?;

    Ok(registry.into())
}

const PATH_POST_MANUAL_ASSOCIATION_MEMBERSHIP_REGISTRY: &str =
    "/account_api/manual_association_membership_registry";

/// Set the manual association membership registry.
///
/// # Access
///
/// Permission [model::Permissions::admin_edit_association_membership] is required.
#[utoipa::path(
    post,
    path = PATH_POST_MANUAL_ASSOCIATION_MEMBERSHIP_REGISTRY,
    request_body = ManualAssociationMembershipRegistryInput,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_manual_association_membership_registry(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(data): Json<ManualAssociationMembershipRegistryInput>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN
        .post_manual_association_membership_registry
        .incr();

    if !permissions.admin_edit_association_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write!(state, move |cmds| {
        cmds.account_admin()
            .association()
            .upsert_manual_registry(data.registry)
            .await
    })?;

    Ok(())
}

const PATH_POST_GET_ASSOCIATION_MEMBERS_PAGE: &str = "/account_api/association_members_page";

/// Get a paged list of association members with an account.
///
/// # Access
///
/// Permission [model::Permissions::admin_view_association_membership] is required.
#[utoipa::path(
    post,
    path = PATH_POST_GET_ASSOCIATION_MEMBERS_PAGE,
    request_body = GetAssociationMembersPage,
    responses(
        (status = 200, description = "Successful.", body = AssociationMembersPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_association_members_page(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(data): Json<GetAssociationMembersPage>,
) -> Result<Json<AssociationMembersPage>, StatusCode> {
    ACCOUNT_ADMIN.post_get_association_members_page.incr();

    if !permissions.admin_view_association_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let entries = state
        .read()
        .account_admin()
        .association()
        .get_page(data)
        .await?;

    Ok(entries.into())
}

const PATH_POST_DELETE_ASSOCIATION_MEMBERSHIP: &str = "/account_api/delete_association_membership";

/// Remove association membership of an account.
///
/// # Access
///
/// Permission [model::Permissions::admin_edit_association_membership] is required.
#[utoipa::path(
    post,
    path = PATH_POST_DELETE_ASSOCIATION_MEMBERSHIP,
    request_body = AccountId,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_delete_association_membership(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(member): Json<AccountId>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.post_delete_association_membership.incr();

    if !permissions.admin_edit_association_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let member = state.get_internal_id(member).await?;

    db_write!(state, move |cmds| {
        cmds.account_admin()
            .association()
            .delete_entry(member)
            .await
    })?;

    Ok(())
}

const PATH_POST_UPDATE_ASSOCIATION_MEMBERSHIP_TYPE: &str =
    "/account_api/update_association_membership_type";

/// Change the membership type of an existing association membership.
///
/// # Access
///
/// Permission [model::Permissions::admin_edit_association_membership] is required.
#[utoipa::path(
    post,
    path = PATH_POST_UPDATE_ASSOCIATION_MEMBERSHIP_TYPE,
    request_body = UpdateAssociationMembershipType,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_update_association_membership_type(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(data): Json<UpdateAssociationMembershipType>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.post_update_association_membership_type.incr();

    if !permissions.admin_edit_association_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let config = state
        .config()
        .client_features_internal()
        .association
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    config
        .membership_types
        .iter()
        .find(|t| t.id == data.membership_type)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let member = state.get_internal_id(data.member).await?;

    db_write!(state, move |cmds| {
        cmds.account_admin()
            .association()
            .update_membership_type(member, data.membership_type)
            .await
    })?;

    Ok(())
}

const PATH_GET_ASSOCIATION_MEMBER: &str = "/account_api/association_member";

/// Get a single association member entry for an account.
///
/// # Access
///
/// Permission [model::Permissions::admin_view_association_membership] is required.
#[utoipa::path(
    post,
    path = PATH_GET_ASSOCIATION_MEMBER,
    request_body = AccountId,
    responses(
        (status = 200, description = "Successful.", body = Option<AssociationMember>),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_association_member(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(member): Json<AccountId>,
) -> Result<Json<Option<AssociationMember>>, StatusCode> {
    ACCOUNT_ADMIN.post_get_association_member.incr();

    if !permissions.admin_view_association_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let member = state.get_internal_id(member).await?;

    let entry = state
        .read()
        .account_admin()
        .association()
        .get_entry(member)
        .await?;

    Ok(entry.into())
}

create_open_api_router!(
    fn router_admin_association,
    get_manual_association_membership_registry,
    post_manual_association_membership_registry,
    post_get_association_members_page,
    post_delete_association_membership,
    post_update_association_membership_type,
    post_get_association_member,
);

create_counters!(
    AccountAdminCounterAssociation,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_ASSOCIATION_COUNTERS_LIST,
    get_manual_association_membership_registry,
    post_manual_association_membership_registry,
    post_get_association_members_page,
    post_delete_association_membership,
    post_update_association_membership_type,
    post_get_association_member,
);
