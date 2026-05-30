use std::collections::HashSet;

use axum::{
    Extension,
    extract::{Path, Query, State},
};
use axum_extra::TypedHeader;
use headers::ContentType;
use model::AdminNotificationTypes;
use model_media::{
    AccountId, AccountIdInternal, AccountState, GetProfileContentQueryParams,
    GetProfileContentResult, GetProfileContentResultInternal, Permissions, ProfileContent,
    SetProfileContent, UpdateProfileContentResult,
};
use server_api::{
    S,
    app::{AdminNotificationProvider, ApiLimitsProvider, ApiUsageTrackerProvider},
    create_open_api_router, db_write,
};
use server_data::read::GetReadCommandsCommon;
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, ReadData, WriteData},
    utils::{Json, StatusCode},
};

async fn read_profile_content_info_result(
    state: &S,
    account_id: AccountIdInternal,
    account_state: AccountState,
    permissions: Permissions,
    requested_profile: AccountIdInternal,
    params: GetProfileContentQueryParams,
) -> Result<GetProfileContentResultInternal, StatusCode> {
    if account_id.as_id() == requested_profile.as_id() {
        return read_profile_content_info_result_for_account(state, requested_profile, params)
            .await;
    }

    if account_state != AccountState::Normal {
        return Ok(GetProfileContentResultInternal::Empty);
    }

    let visibility = state
        .read()
        .common()
        .account(requested_profile)
        .await?
        .is_profile_visible();

    if visibility
        || permissions.admin_view_all_profiles
        || (params.allow_get_content_if_match()
            && state
                .data_all_access()
                .is_match(account_id, requested_profile)
                .await?)
    {
        read_profile_content_info_result_for_account(state, requested_profile, params).await
    } else {
        Ok(GetProfileContentResultInternal::Empty)
    }
}

async fn read_profile_content_info_result_for_account(
    state: &S,
    requested_profile: AccountIdInternal,
    params: GetProfileContentQueryParams,
) -> Result<GetProfileContentResultInternal, StatusCode> {
    let internal = state
        .read()
        .media()
        .current_account_media(requested_profile)
        .await?;

    let info: ProfileContent = internal.clone().into();

    Ok(match params.version() {
        Some(param_version) if param_version == internal.profile_content_version_uuid => {
            GetProfileContentResultInternal::VersionOnly(internal.profile_content_version_uuid)
        }
        _ => GetProfileContentResultInternal::ContentWithVersion {
            content: info,
            version: internal.profile_content_version_uuid,
        },
    })
}

const PATH_GET_PROFILE_CONTENT_INFO: &str = "/media_api/profile_content_info/{aid}";

/// Get current profile content for selected profile.
///
/// # Access
///
/// ## Own profile
/// Unrestricted access.
///
/// ## Other profiles
/// Normal account state required.
///
/// ## Private other profiles
/// If the profile is a match, then the profile can be accessed if query
/// parameter `is_match` is set to `true`.
///
/// If the profile is not a match, then permission `admin_view_all_profiles`
/// is required.
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_CONTENT_INFO,
    params(AccountId, GetProfileContentQueryParams),
    responses(
        (status = 200, description = "Get profile content info.", body = GetProfileContentResult),
        (status = 401, description = "Unauthorized."),
        (status = 429, description = "Too many requests."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_content_info(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Extension(permissions): Extension<Permissions>,
    Path(requested_profile): Path<AccountId>,
    Query(params): Query<GetProfileContentQueryParams>,
) -> Result<Json<GetProfileContentResult>, StatusCode> {
    MEDIA.get_profile_content_info.incr();
    state
        .api_usage_tracker()
        .incr(account_id, |u| &u.get_profile_content_info)
        .await;
    state
        .api_limits(account_id)
        .media()
        .get_profile_content_info()
        .await?;

    let requested_profile = state.get_internal_id(requested_profile).await?;
    let result = read_profile_content_info_result(
        &state,
        account_id,
        account_state,
        permissions,
        requested_profile,
        params,
    )
    .await?;

    Ok(GetProfileContentResult::from(result).into())
}

const PATH_GET_PROFILE_CONTENT_INFO_BINARY: &str = "/media_api/profile_content_info_binary/{aid}";

/// Get current profile content for selected profile as compact binary payload.
///
/// The first byte is result variant:
/// - 0 = Empty
/// - 1 = VersionOnly
/// - 2 = ContentWithVersion
///
/// Variant payloads:
/// - Empty: no payload
/// - VersionOnly: 16-byte profile content version UUID
/// - ContentWithVersion:
///   - 16-byte profile content version UUID
///   - 1-byte verification status (low 8 bits of internal flags)
///   - 1-byte content count (max 6)
///   - repeated content entries:
///     - 16-byte content UUID
///     - 1-byte packed content info
///   - 4-byte crop size as little-endian f32
///   - 4-byte crop x as little-endian f32
///   - 4-byte crop y as little-endian f32
///
/// Packed content info byte layout:
/// - bits 0..2: face verified (0 None, 1 false, 2 true)
/// - bit 3: face detected
/// - bit 4: accepted
/// - bits 5..7: media content type
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_CONTENT_INFO_BINARY,
    params(AccountId, GetProfileContentQueryParams),
    responses(
        (status = 200, description = "Get profile content info as binary.", body = inline(model::BinaryData), content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized."),
        (status = 429, description = "Too many requests."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_content_info_binary(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Extension(permissions): Extension<Permissions>,
    Path(requested_profile): Path<AccountId>,
    Query(params): Query<GetProfileContentQueryParams>,
) -> Result<(TypedHeader<ContentType>, Vec<u8>), StatusCode> {
    MEDIA.get_profile_content_info_binary.incr();
    state
        .api_usage_tracker()
        .incr(account_id, |u| &u.get_profile_content_info)
        .await;
    state
        .api_limits(account_id)
        .media()
        .get_profile_content_info()
        .await?;

    let requested_profile = state.get_internal_id(requested_profile).await?;
    let result = read_profile_content_info_result(
        &state,
        account_id,
        account_state,
        permissions,
        requested_profile,
        params,
    )
    .await?;

    Ok((TypedHeader(ContentType::octet_stream()), result.to_binary()))
}

const PATH_PUT_PROFILE_CONTENT: &str = "/media_api/profile_content";

/// Set new profile content for current account.
///
/// This also moves the content to moderation if it is not already
/// in moderation or moderated.
///
/// Also profile visibility moves from pending to normal when
/// all profile content is moderated as accepted.
///
/// # Restrictions
/// - All content must be owned by the account.
/// - All content must be images.
#[utoipa::path(
    put,
    path = PATH_PUT_PROFILE_CONTENT,
    request_body(content = SetProfileContent),
    responses(
        (status = 200, description = "Successful.", body = UpdateProfileContentResult),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("access_token" = [])),
)]
pub async fn put_profile_content(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Json(new): Json<SetProfileContent>,
) -> Result<Json<UpdateProfileContentResult>, StatusCode> {
    MEDIA.put_profile_content.incr();

    let account_content = state
        .read()
        .media()
        .all_account_media_content(api_caller_account_id)
        .await?;
    let available_content_ids: HashSet<_> =
        account_content.iter().map(|v| v.content_id()).collect();
    if let Some(missing_index) =
        new.content
            .iter()
            .take(6)
            .enumerate()
            .find_map(|(index, content_id)| {
                (!available_content_ids.contains(content_id)).then_some(index as i64)
            })
    {
        return Ok(UpdateProfileContentResult::error_content_does_not_exist(missing_index).into());
    }

    db_write!(state, move |cmds| {
        cmds.media()
            .update_profile_content(api_caller_account_id, new)
            .await
    })?;

    state
        .admin_notification()
        .send_notification_if_needed(AdminNotificationTypes::ModerateInitialMediaContentBot)
        .await;
    state
        .admin_notification()
        .send_notification_if_needed(AdminNotificationTypes::ModerateMediaContentBot)
        .await;

    Ok(UpdateProfileContentResult::success().into())
}

create_open_api_router!(
    fn router_profile_content,
    get_profile_content_info,
    get_profile_content_info_binary,
    put_profile_content,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_PROFILE_CONTENT_COUNTERS_LIST,
    get_profile_content_info,
    get_profile_content_info_binary,
    put_profile_content,
);
