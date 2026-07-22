use axum::{Extension, extract::State};
use model::{AccountId, Permissions};
use model_account::{
    AssociationMemberIdManual, AssociationMemberManual, AssociationMembersPage,
    EditAssociationMemberManual, GetAssociationMembersPage, NewAssociationMemberManual,
};
use server_api::{
    S,
    app::{GetAccounts, ReadData},
    create_open_api_router, db_write,
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use crate::{
    app::WriteData,
    utils::{Json, StatusCode},
};

const PATH_GET_ALL_ASSOCIATION_MEMBERS_MANUAL: &str = "/account_api/association_members_manual";

/// Get all manually added association members.
///
/// # Access
///
/// Permission [model::Permissions::admin_view_association_membership] is required.
#[utoipa::path(
    get,
    path = PATH_GET_ALL_ASSOCIATION_MEMBERS_MANUAL,
    responses(
        (status = 200, description = "Successful.", body = Vec<AssociationMemberManual>),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_all_association_members_manual(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<Vec<AssociationMemberManual>>, StatusCode> {
    ACCOUNT_ADMIN.get_all_association_members_manual.incr();

    if !permissions.admin_view_association_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let entries = state
        .read()
        .account_admin()
        .association()
        .get_all_manual()
        .await?;

    Ok(entries.into())
}

const PATH_POST_ADD_ASSOCIATION_MEMBER_MANUAL: &str = "/account_api/add_association_member_manual";

/// Add a new manual association member.
///
/// # Access
///
/// Permission [model::Permissions::admin_edit_association_membership] is required.
#[utoipa::path(
    post,
    path = PATH_POST_ADD_ASSOCIATION_MEMBER_MANUAL,
    request_body = NewAssociationMemberManual,
    responses(
        (status = 200, description = "Successful.", body = AssociationMemberIdManual),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_add_association_member_manual(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(account_id): Extension<model::AccountIdInternal>,
    Json(data): Json<NewAssociationMemberManual>,
) -> Result<Json<AssociationMemberIdManual>, StatusCode> {
    ACCOUNT_ADMIN.post_add_association_member_manual.incr();

    if !permissions.admin_edit_association_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let entry_id = db_write!(state, move |cmds| {
        cmds.account_admin()
            .association()
            .create_entry_manual(
                account_id,
                data.full_name,
                data.domicile,
                data.email,
                data.membership_type,
            )
            .await
    })?;

    Ok(entry_id.into())
}

const PATH_POST_EDIT_ASSOCIATION_MEMBER_MANUAL: &str =
    "/account_api/edit_association_member_manual";

/// Edit manual association member.
///
/// # Access
///
/// Permission [model::Permissions::admin_edit_association_membership] is required.
#[utoipa::path(
    post,
    path = PATH_POST_EDIT_ASSOCIATION_MEMBER_MANUAL,
    request_body = EditAssociationMemberManual,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_edit_association_member_manual(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(account_id): Extension<model::AccountIdInternal>,
    Json(data): Json<EditAssociationMemberManual>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.post_edit_association_member_manual.incr();

    if !permissions.admin_edit_association_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write!(state, move |cmds| {
        cmds.account_admin()
            .association()
            .edit_entry_manual(
                account_id,
                data.id,
                data.full_name,
                data.domicile,
                data.email,
                data.membership_type,
            )
            .await
    })?;

    Ok(())
}

const PATH_DELETE_ASSOCIATION_MEMBER_MANUAL: &str = "/account_api/delete_association_member_manual";

/// Delete a manual association member.
///
/// # Access
///
/// Permission [model::Permissions::admin_edit_association_membership] is required.
#[utoipa::path(
    delete,
    path = PATH_DELETE_ASSOCIATION_MEMBER_MANUAL,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_association_member_manual(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(data): Json<AssociationMemberIdManual>,
) -> Result<(), StatusCode> {
    ACCOUNT_ADMIN.delete_association_member_manual.incr();

    if !permissions.admin_edit_association_membership {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write!(state, move |cmds| {
        cmds.account_admin()
            .association()
            .delete_entry_manual(data)
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

create_open_api_router!(
    fn router_admin_association,
    get_all_association_members_manual,
    post_add_association_member_manual,
    post_edit_association_member_manual,
    delete_association_member_manual,
    post_get_association_members_page,
    post_delete_association_membership,
);

create_counters!(
    AccountAdminCounterAssociation,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_ASSOCIATION_COUNTERS_LIST,
    get_all_association_members_manual,
    post_add_association_member_manual,
    post_edit_association_member_manual,
    delete_association_member_manual,
    post_get_association_members_page,
    post_delete_association_membership,
);
