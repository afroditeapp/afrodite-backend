#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! HTTP API types and request handlers for all servers.

use utoipa::OpenApi;

use self::utils::SecurityApiAccessTokenDefault;

// Routes
pub mod account;
pub mod account_internal;

pub use server_api::{app, internal_api, utils};
pub use server_common::{data::DataError, result};

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        // Account
        account::post_sign_in_with_login,
        account::post_account_setup,
        account::post_account_data,
        account::post_complete_setup,
        account::post_delete,
        account::post_demo_mode_login,
        account::post_demo_mode_confirm_login,
        account::post_demo_mode_register_account,
        account::post_demo_mode_login_to_account,
        account::post_demo_mode_accessible_accounts,
        account::put_setting_profile_visiblity,
        account::put_setting_unlimited_likes,
        account::get_account_state,
        account::get_account_setup,
        account::get_account_data,
        account::get_latest_birthdate,
        account::get_deletion_status,
        account::delete_cancel_deletion,
        account::post_get_next_client_id,
        account::post_get_news_count,
        account::post_reset_news_paging,
        account::post_get_next_news_page,
        // Account internal
        account_internal::post_register,
        account_internal::post_login,
    ),
    components(schemas(
        // Account
        model::account::Account,
        model::account::AccountState,
        model::account::AccountSetup,
        model::account::SetAccountSetup,
        model::account::AccountData,
        model::account::Capabilities,
        model::account::BooleanSetting,
        model::account::DeleteStatus,
        model::account::SignInWithLoginInfo,
        model::account::LoginResult,
        model::account::AuthPair,
        model::account::ProfileVisibility,
        model::account::AccessibleAccount,
        model::account::DemoModePassword,
        model::account::DemoModeToken,
        model::account::DemoModeLoginResult,
        model::account::DemoModeLoginToken,
        model::account::DemoModeLoginToAccount,
        model::account::DemoModeConfirmLogin,
        model::account::DemoModeConfirmLoginResult,
        model::account::EmailAddress,
        model::account::LatestBirthdate,
        model::account::ClientId,
        model::account::ClientLocalId,
        model::account::ClientType,
        model::account::ClientInfo,
        model::account::NewsId,
        model::account::NewsIteratorSessionId,
        model::account::NewsCount,
        model::account::NewsCountResult,
        model::account::ResetNewsIteratorResult,
        model::account::NewsPage,
        model::account::NewsItemSimple,
        model::account::NewsItem,
        model::account::NewsSyncVersion,
    )),
    modifiers(&SecurityApiAccessTokenDefault),
)]
pub struct ApiDocAccount;

pub use server_api::{db_write, db_write_multiple};
