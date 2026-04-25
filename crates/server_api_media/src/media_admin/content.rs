use axum::{Extension, extract::State};
use model::AdminBotNotificationTypes;
use model_media::{
    AccountIdInternal, GetMediaContentFaceVerifiedNullList, Permissions,
    PostMediaContentFaceDetectedValue, PostMediaContentFaceVerifiedValue,
};
use server_api::{
    DataError, S,
    app::{AdminNotificationProvider, GetAccounts, ReadData},
    create_open_api_router,
    result::WrappedContextExt,
};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use crate::{
    app::WriteData,
    db_write,
    utils::{Json, StatusCode},
};

const PATH_POST_MEDIA_CONTENT_FACE_DETECTED_VALUE: &str =
    "/media_api/media_content_face_detected_value";

/// Change media content face detected value
///
/// # Access
/// * Permission [model::Permissions::admin_edit_media_content_face_detected_value]
#[utoipa::path(
    post,
    path = PATH_POST_MEDIA_CONTENT_FACE_DETECTED_VALUE,
    request_body = PostMediaContentFaceDetectedValue,
    responses(
        (status = 200, description = "Successful"),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn post_media_content_face_detected_value(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(data): Json<PostMediaContentFaceDetectedValue>,
) -> Result<(), StatusCode> {
    MEDIA_ADMIN.post_media_content_face_detected_value.incr();

    if !permissions.admin_edit_media_content_face_detected_value {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let content_owner = state.get_internal_id(data.account_id).await?;

    let changed = db_write!(state, move |cmds| {
        let content_id = cmds
            .read()
            .media()
            .content_id_internal(content_owner, data.content_id)
            .await?;
        cmds.media_admin()
            .content()
            .change_face_detected_value(content_id, data.value)
            .await
    })?;

    if changed {
        state
            .admin_notification()
            .send_bot_notification_if_needed(
                AdminBotNotificationTypes::VERIFY_MEDIA_CONTENT_FACE_BOT,
            )
            .await;
    }

    Ok(())
}

const PATH_GET_MEDIA_CONTENT_FACE_VERIFIED_NULL_LIST: &str =
    "/media_api/media_content_face_verified_null_list";

/// Get first page of accounts with security selfie and content where `face_verified` is NULL
/// and `face_detected` is true or `face_detected_manual` is true.
/// Oldest security content set time is first and count 25.
#[utoipa::path(
    get,
    path = PATH_GET_MEDIA_CONTENT_FACE_VERIFIED_NULL_LIST,
    responses(
        (status = 200, description = "Successful", body = GetMediaContentFaceVerifiedNullList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_media_content_face_verified_null_list(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetMediaContentFaceVerifiedNullList>, StatusCode> {
    MEDIA_ADMIN.get_media_content_face_verified_null_list.incr();

    if !permissions.admin_edit_media_content_face_verified_value {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let values = state
        .read()
        .media_admin()
        .media_content_face_verified_null_list()
        .await?;

    Ok(values.into())
}

const PATH_POST_MEDIA_CONTENT_FACE_VERIFIED_VALUE: &str =
    "/media_api/media_content_face_verified_value";

/// Change media content face verified value
///
/// Bot account sets automatic value and human admin account sets manual override value.
///
/// # Access
/// * Permission [model::Permissions::admin_edit_media_content_face_verified_value]
#[utoipa::path(
    post,
    path = PATH_POST_MEDIA_CONTENT_FACE_VERIFIED_VALUE,
    request_body = PostMediaContentFaceVerifiedValue,
    responses(
        (status = 200, description = "Successful"),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn post_media_content_face_verified_value(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<PostMediaContentFaceVerifiedValue>,
) -> Result<(), StatusCode> {
    MEDIA_ADMIN.post_media_content_face_verified_value.incr();

    if !permissions.admin_edit_media_content_face_verified_value {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if data.values.is_empty() {
        return Ok(());
    }

    let content_owner = state.get_internal_id(data.account_id).await?;

    db_write!(state, move |cmds| {
        let current_security_content = cmds
            .read()
            .media()
            .current_account_media(content_owner)
            .await?
            .security_content_id
            .map(|v| v.content_id());

        if current_security_content != Some(data.security_content) {
            return Err(DataError::NotAllowed.report());
        }

        let mut values = Vec::with_capacity(data.values.len());
        for value in data.values {
            let content_id = cmds
                .read()
                .media()
                .content_id_internal(content_owner, value.content_id)
                .await?;
            values.push((content_id, value.value));
        }

        cmds.media_admin()
            .content()
            .change_face_verified_values(moderator_id, values)
            .await?;

        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(
        fn router_admin_content,
        post_media_content_face_detected_value,
        get_media_content_face_verified_null_list,
        post_media_content_face_verified_value,
);

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_CONTENT_COUNTERS_LIST,
    post_media_content_face_detected_value,
    get_media_content_face_verified_null_list,
    post_media_content_face_verified_value,
);
