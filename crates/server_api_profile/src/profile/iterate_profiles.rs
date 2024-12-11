use axum::{extract::State, Extension};
use model_profile::{AccountIdInternal, ProfileIteratorSessionId, ProfilePage};
use obfuscate_api_macro::obfuscate_api;
use server_api::{create_open_api_router, S};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::WriteData,
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_POST_NEXT_PROFILE_PAGE: &str = "/profile_api/page/next";

/// Post (updates iterator) to get next page of profile list.
#[utoipa::path(
    post,
    path = PATH_POST_NEXT_PROFILE_PAGE,
    request_body(content = ProfileIteratorSessionId),
    responses(
        (status = 200, description = "Update successfull.", body = ProfilePage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_next_profile_page(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(iterator_session_id): Json<ProfileIteratorSessionId>,
) -> Result<Json<ProfilePage>, StatusCode> {
    PROFILE.post_get_next_profile_page.incr();

    let data = state
        .concurrent_write_profile_blocking(account_id.as_id(), move |cmds| {
            cmds.next_profiles(account_id, iterator_session_id)
        })
        .await??;

    if let Some(data) = data {
        // Profile iterator session ID was valid
        Ok(ProfilePage {
            profiles: data,
            error_invalid_iterator_session_id: false,
        }
        .into())
    } else {
        Ok(ProfilePage {
            profiles: vec![],
            error_invalid_iterator_session_id: true,
        }
        .into())
    }
}

// TODO(prod): Consider adding support for optional random iterator initial
//             position and max distance. Could filtering settings
//             include those?

#[obfuscate_api]
const PATH_POST_RESET_PROFILE_PAGING: &str = "/profile_api/page/reset";

/// Reset profile paging.
///
/// After this request getting next profiles will continue from the nearest
/// profiles.
#[utoipa::path(
    post,
    path = PATH_POST_RESET_PROFILE_PAGING,
    responses(
        (status = 200, description = "Update successfull.", body = ProfileIteratorSessionId),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_reset_profile_paging(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ProfileIteratorSessionId>, StatusCode> {
    PROFILE.post_reset_profile_paging.incr();
    let iterator_session_id: ProfileIteratorSessionId = state
        .concurrent_write_profile_blocking(account_id.as_id(), move |cmds| {
            cmds.reset_profile_iterator(account_id)
        })
        .await??
        .into();

    Ok(iterator_session_id.into())
}

pub fn iterate_profiles_router(s: S) -> OpenApiRouter {
    create_open_api_router!(s, post_get_next_profile_page, post_reset_profile_paging,)
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ITERATE_PROFILES_COUNTERS_LIST,
    post_get_next_profile_page,
    post_reset_profile_paging,
);
