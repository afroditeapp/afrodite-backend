use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, ProfileLink, ProfilePage};
use simple_backend::create_counters;

use crate::{
    api::utils::{Json, StatusCode},
    app::{GetAccessTokens, ReadData, WriteData},
    data::{
        write_concurrent::{ConcurrentWriteAction, ConcurrentWriteProfileHandle},
        DataError,
    },
};

pub const PATH_POST_NEXT_PROFILE_PAGE: &str = "/profile_api/page/next";

/// Post (updates iterator) to get next page of profile list.
#[utoipa::path(
    post,
    path = "/profile_api/page/next",
    responses(
        (status = 200, description = "Update successfull.", body = ProfilePage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_next_profile_page<S: GetAccessTokens + WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ProfilePage>, StatusCode> {
    PROFILE.post_get_next_profile_page.incr();

    let data = state
        .write_concurrent(account_id.as_id(), move |cmds| async move {
            let out: ConcurrentWriteAction<crate::result::Result<Vec<ProfileLink>, DataError>> =
                cmds.accquire_profile(move |cmds: ConcurrentWriteProfileHandle| {
                    Box::new(async move { cmds.next_profiles(account_id).await })
                })
                .await;
            out
        })
        .await??;

    Ok(ProfilePage { profiles: data }.into())
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
        (status = 200, description = "Update successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_reset_profile_paging<S: GetAccessTokens + WriteData + ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<(), StatusCode> {
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

    Ok(())
}

pub fn iterate_profiles_router(s: crate::app::S) -> Router {
    use axum::routing::post;

    use crate::app::S;

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
