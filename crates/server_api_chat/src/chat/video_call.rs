use axum::{
    Extension,
    extract::{Query, State},
};
use model::AccountId;
use model_chat::{AccountIdInternal, JitsiMeetUrl, PostVideoCallUrlResult};
use server_api::{
    S,
    app::{ApiUsageTrackerProvider, GetAccounts, ReadData, WriteData},
    create_open_api_router,
};
use server_data::read::GetReadCommandsCommon;
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::{
    app::JitsiMeetUrlCreatorProvider, create_counters, jitsi_meet::VideoCallUserInfo,
};

use super::super::utils::{Json, StatusCode};
use crate::db_write;

const PATH_POST_CREATE_VIDEO_CALL_URL: &str = "/chat_api/post_create_video_call_url";

/// Create video call URL to a meeting with an user.
///
/// The user must be a match.
///
/// If result value is empty then video calling is disabled.
#[utoipa::path(
    post,
    path = PATH_POST_CREATE_VIDEO_CALL_URL,
    params(AccountId),
    responses(
        (status = 200, description = "Success.", body = PostVideoCallUrlResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_create_video_call_url(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Query(other_user): Query<AccountId>,
) -> Result<Json<PostVideoCallUrlResult>, StatusCode> {
    CHAT.post_create_video_call_url.incr();
    state
        .api_usage_tracker()
        .incr(id, |u| &u.post_create_video_call_url)
        .await;

    let other_user = state.get_internal_id(other_user).await?;

    let is_match = state
        .read()
        .chat()
        .account_interaction(id, other_user)
        .await?
        .map(|v| v.is_match())
        .unwrap_or_default();

    if !is_match {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let name = state
        .read()
        .common()
        .user_visible_profile_name_if_data_available(id)
        .await?;

    let other_user_name = state
        .read()
        .common()
        .user_visible_profile_name_if_data_available(other_user)
        .await?;

    let urls = state.jitsi_meet_url_creator().create_url(
        VideoCallUserInfo {
            id: id.as_id().to_string(),
            name: name
                .map(|v| v.into_string())
                .unwrap_or_else(|| "Caller".to_string()),
        },
        VideoCallUserInfo {
            id: other_user.as_id().to_string(),
            name: other_user_name
                .map(|v| v.into_string())
                .unwrap_or_else(|| "Callee".to_string()),
        },
    )?;

    if urls.is_some() {
        let already_created = state
            .read()
            .chat()
            .is_video_call_url_already_created(id, other_user)
            .await?;

        if !already_created {
            db_write!(state, move |cmds| {
                cmds.chat()
                    .mark_video_call_url_created(id, other_user)
                    .await
            })?;
        }
    }

    let r = urls
        .map(|urls| PostVideoCallUrlResult {
            jitsi_meet: Some(JitsiMeetUrl {
                url: urls.url,
                custom_url: urls.custom_url,
            }),
        })
        .unwrap_or_default();

    Ok(r.into())
}

create_open_api_router!(fn router_video_call, post_create_video_call_url,);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_VIDEO_CALL_COUNTERS_LIST,
    post_create_video_call_url,
);
