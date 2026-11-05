#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Account API tests
//!
//! Tests for `account_api` and `account_admin_api` routes in the
//! `server_api_account` crate.
//!
//! Test modules mirror the server route module layout:
//!
//! * `account/<module>/<api_name>.rs` - tests for
//!   `server_api_account::account` routes, e.g. tests for
//!   `crates/server_api_account/src/account/state.rs` are in
//!   `account/state/<api_name>.rs` where `<api_name>` is the server side
//!   handler function name
//! * `account_admin/<module>/<api_name>.rs` - tests for
//!   `server_api_account::account_admin` routes
//!
//! All `server_api_account::account` routes which require an access token
//! have tests. Public routes (`post_sign_in_with_login`,
//! `post_request_email_login_token`, `post_email_login_with_token`,
//! `post_verify_app_attestation`, `post_request_app_attest_challenge`,
//! `get_email_registration_platforms`, demo account APIs, email token
//! verification pages `get_verify_email`/`post_verify_email`/
//! `get_verify_new_email`/`post_verify_new_email`) do not have
//! authorization tests because they do not require an access token. Demo
//! account APIs are covered indirectly by `Account::register_and_login`
//! which uses the same registration machinery.
//!
//! APIs which require email sending are not tested: email sending relies
//! on an SMTP server configuration which is not available in the test
//! environment. Untested APIs are `post_send_verify_email_message`,
//! `post_cancel_email_change`, `post_init_email_change`,
//! `post_initial_email`, `post_set_email_login_enabled`,
//! `get_email_address_state`, `get_email_address_state_admin`,
//! `post_admin_cancel_email_change` and `post_admin_init_email_change`.
//!
//! Custom report APIs (`post_custom_report_empty`,
//! `post_custom_report_string`, `post_get_custom_reports_config`) are not
//! tested because custom reports are not configured in the test server
//! config.
//!
//! APIs which require external service token validation are tested for the
//! unauthorized request and disassociate cases: `put_sign_in_with_apple`
//! and `put_sign_in_with_google` (Apple / Google token validation is not
//! available in the test environment).
//!
//! Admin routes are tested with `request_without_admin_permission_fails`
//! test using an account without admin permissions, together with a
//! successful admin account test.
//!
//! `account_admin` APIs without a positive-path test in a dedicated test
//! file:
//!
//! - `custom_email`
//!   - get_custom_email_config
//!   - get_custom_email_list
//!   - post_send_custom_email_to_all_accounts
//!
//! (`get_custom_email_config` and `get_custom_email_list` only return
//! static config data. `post_send_custom_email_to_all_accounts` and
//! positive-path tests for `post_create_custom_email`,
//! `post_update_custom_email` and
//! `post_send_custom_email_draft_to_my_email_address` send emails via
//! SMTP when exercised with an admin account, so those APIs are covered
//! only by the permission check tests in `account_admin/custom_email.rs`)
//!
//! APIs without any tests because a related feature is not enabled in the
//! test server client features config:
//!
//! - association membership (`account/association.rs`,
//!   `account_admin/association.rs`)
//!
//! Original (pre-suite-generation) tests are in `original/`.

pub(crate) mod account;
pub(crate) mod account_admin;
mod original;

pub fn call_this_to_make_sure_that_crate_is_linked() {}
