use axum::{Extension, extract::State};
use model::Permissions;
use model_profile::{ProfileAttributesSchemaExport, UpdateProfileAttributesSchema};
use server_api::{S, create_open_api_router, db_write};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;

use crate::{
    app::{ReadData, WriteData},
    utils::{Json, StatusCode},
};

const PATH_PROFILE_ATTRIBUTES_SCHEMA: &str = "/profile_api/profile_attributes_schema";

/// Get profile attributes schema from DB.
///
/// # Access
/// - Permission [Permissions::admin_edit_profile_attributes_schema].
#[utoipa::path(
    get,
    path = PATH_PROFILE_ATTRIBUTES_SCHEMA,
    responses(
        (status = 200, description = "Successful.", body = ProfileAttributesSchemaExport),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_attributes_schema(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<ProfileAttributesSchemaExport>, StatusCode> {
    PROFILE.get_profile_attributes_schema.incr();

    if !permissions.admin_edit_profile_attributes_schema {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let schema = state
        .read()
        .profile_admin()
        .attribute_schema()
        .get_schema()
        .await?;

    Ok(schema.into())
}

/// Add or edit profile attributes to profile attributes schema in DB.
///
/// Removing attributes or attribute values is not possible.
///
/// # Access
/// - Permission [Permissions::admin_edit_profile_attributes_schema].
/// - Modifying user visible values (texts and icons) requires
///   [Permissions::admin_edit_profile_attributes_schema_visible_content].
#[utoipa::path(
    put,
    path = PATH_PROFILE_ATTRIBUTES_SCHEMA,
    request_body = UpdateProfileAttributesSchema,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_profile_attributes_schema(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(request): Json<UpdateProfileAttributesSchema>,
) -> Result<(), StatusCode> {
    PROFILE.put_profile_attributes_schema.incr();

    if !permissions.admin_edit_profile_attributes_schema {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let has_content_permission = permissions.admin_edit_profile_attributes_schema_visible_content;

    db_write!(state, move |cmds| {
        cmds.profile_admin()
            .attribute_schema()
            .update_schema(request, has_content_permission)
            .await
    })?;

    Ok(())
}

create_open_api_router!(
    fn router_admin_profile_attributes,
    get_profile_attributes_schema,
    put_profile_attributes_schema,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_PROFILE_ATTRIBUTES_COUNTERS_LIST,
    get_profile_attributes_schema,
    put_profile_attributes_schema,
);
