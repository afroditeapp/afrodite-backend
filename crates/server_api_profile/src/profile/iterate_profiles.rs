use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, IteratorSessionId, ProfilePage};
use server_api::db_write;
use server_data::write_concurrent::{ConcurrentWriteAction, ConcurrentWriteProfileHandle};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;

use crate::{
    app::{GetAccessTokens, ReadData, StateBase, WriteData},
    utils::{Json, StatusCode},
    DataError,
};

pub const PATH_POST_NEXT_PROFILE_PAGE: &str = "/profile_api/page/next";

/// Post (updates iterator) to get next page of profile list.
#[utoipa::path(
    post,
    path = "/profile_api/page/next",
    request_body(content = IteratorSessionId),
    responses(
        (status = 200, description = "Update successfull.", body = ProfilePage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_next_profile_page<S: GetAccessTokens + WriteData + ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(iterator_session_id): Json<IteratorSessionId>,
) -> Result<Json<ProfilePage>, StatusCode> {
    PROFILE.post_get_next_profile_page.incr();

    let current_iterator_session_id: Option<IteratorSessionId> = state
        .read()
        .profile()
        .profile_iterator_session_id(account_id)
        .await?
        .map(|v| v.into());

    if current_iterator_session_id == Some(iterator_session_id) {
        let data = state
            .concurrent_write_blocking(
                account_id.as_id(),
                move |cmds| {
                    cmds.profile_blocking().next_profiles(account_id)
                }
            )
            .await??;

        Ok(ProfilePage {
            profiles: data,
            error_invalid_iterator_session_id: false,
        }.into())
    } else {
        Ok(ProfilePage {
            profiles: vec![],
            error_invalid_iterator_session_id: true,
        }.into())
    }
}

pub const PATH_POST_RESET_PROFILE_PAGING: &str = "/profile_api/page/reset";

/// Reset profile paging.
///
/// After this request getting next profiles will continue from the nearest
/// profiles.
#[utoipa::path(
    post,
    path = "/profile_api/page/reset",
    responses(
        (status = 200, description = "Update successfull.", body = IteratorSessionId),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_reset_profile_paging<S: GetAccessTokens + WriteData + ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<IteratorSessionId>, StatusCode> {
    PROFILE.post_reset_profile_paging.incr();
    state
        .write_concurrent(account_id.as_id(), move |cmds| async move {
            let out: ConcurrentWriteAction<crate::result::Result<_, DataError>> = cmds
                .accquire_profile(move |cmds: ConcurrentWriteProfileHandle| {
                    Box::new(async move { cmds.reset_profile_iterator(account_id).await })
                })
                .await;
            out
        })
        .await??;

    let iterator_session_id: IteratorSessionId = db_write!(state, move |cmds|
        cmds.profile().update_profile_iterator_session_id(account_id)
    )?.into();

    Ok(iterator_session_id.into())
}

pub fn iterate_profiles_router<S: StateBase + GetAccessTokens + WriteData + ReadData>(
    s: S,
) -> Router {
    use axum::routing::post;

    Router::new()
        .route(
            PATH_POST_NEXT_PROFILE_PAGE,
            post(post_get_next_profile_page::<S>),
        )
        .route(
            PATH_POST_RESET_PROFILE_PAGING,
            post(post_reset_profile_paging::<S>),
        )
        .with_state(s)
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ITERATE_PROFILES_COUNTERS_LIST,
    post_get_next_profile_page,
    post_reset_profile_paging,
);
