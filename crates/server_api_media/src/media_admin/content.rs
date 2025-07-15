use axum::{Extension, extract::State};
use model_media::{EventToClientInternal, Permissions, PostMediaContentFaceDetectedValue};
use server_api::{S, app::GetAccounts, create_open_api_router};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use crate::{
    app::WriteData,
    db_write,
    utils::{Json, StatusCode},
};

// TODO(prod): Remove /admin/ from all paths for consistensy

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

    db_write!(state, move |cmds| {
        let content_id = cmds
            .read()
            .media()
            .content_id_internal(content_owner, data.content_id)
            .await?;
        cmds.media_admin()
            .content()
            .change_face_detected_value(content_id, data.value)
            .await?;

        cmds.events()
            .send_connected_event(
                content_id.content_owner(),
                EventToClientInternal::MediaContentChanged,
            )
            .await?;

        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(
        fn router_admin_content,
        post_media_content_face_detected_value,
);

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_CONTENT_COUNTERS_LIST,
    post_media_content_face_detected_value,
);
