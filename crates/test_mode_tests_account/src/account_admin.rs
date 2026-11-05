//! Tests for APIs in `server_api_account::account_admin`
//!
//! APIs which do not have tests:
//!
//! - Email address admin APIs (`account_admin/email.rs`) are not tested
//!   because email verification state and SMTP based sending depend on
//!   server email configuration which is not available in the test
//!   environment.
//! - Association membership admin APIs are not tested because the
//!   association feature is not enabled in the test server client features
//!   config.
//!
//! `client_version` APIs do not have a test file:
//!
//! - post_get_client_version_statistics
//!
//! (API is a read-only statistics query which does not modify state, and
//! the permission check pattern is the same as in the other admin test
//! files)

pub(crate) mod ban;
pub(crate) mod custom_email;
pub(crate) mod delete;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod news;
pub(crate) mod permissions;
pub(crate) mod search;
pub(crate) mod state;
pub(crate) mod verification;
