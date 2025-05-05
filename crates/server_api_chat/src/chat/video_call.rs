use axum::{
    extract::{Query, State}, Extension
};
use model::AccountId;
use model_chat::{
    AccountIdInternal, GetVideoCallUrlsResult
};
use server_api::{
    app::{ReadData, GetAccounts}, create_open_api_router, S
};
use server_data_chat::read::GetReadChatCommands;
use simple_backend::{app::JitsiMeetUrlCreatorProvider, create_counters, jitsi_meet::VideoCallUserInfo};
use server_data::read::GetReadCommandsCommon;

use super::super::utils::{Json, StatusCode};

const PATH_GET_VIDEO_CALL_URLS: &str = "/chat_api/get_video_call_urls";

/// Create Jitsi Meet video call URLs to a meeting with an user.
///
/// The user must be a match.
#[utoipa::path(
    get,
    path = PATH_GET_VIDEO_CALL_URLS,
    params(AccountId),
    responses(
        (status = 200, description = "Success.", body = GetVideoCallUrlsResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_video_call_urls(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Query(other_user): Query<AccountId>,
) -> Result<Json<GetVideoCallUrlsResult>, StatusCode> {
    CHAT.get_video_call_urls.incr();

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
        .get_profile_age_and_name_if_profile_component_is_enabled(id)
        .await?
        .map(|v| v.name);

    let other_user_name = state
        .read()
        .common()
        .get_profile_age_and_name_if_profile_component_is_enabled(other_user)
        .await?
        .map(|v| v.name);


    let urls = state.jitsi_meet_url_creator().create_url(
        VideoCallUserInfo {
            id: id.as_id().to_string(),
            name: name.unwrap_or_else(|| "Caller".to_string()),
        },
        VideoCallUserInfo {
            id: other_user.as_id().to_string(),
            name: other_user_name.unwrap_or_else(|| "Callee".to_string()),
        },
    )?;

    Ok(GetVideoCallUrlsResult {
        url: urls.url,
        custom_url: urls.custom_url
    }.into())
}

create_open_api_router!(fn router_video_call, get_video_call_urls,);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_VIDEO_CALL_COUNTERS_LIST,
    get_video_call_urls,
);
