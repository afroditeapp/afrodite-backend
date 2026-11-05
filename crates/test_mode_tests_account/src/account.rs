//! Tests for APIs in `server_api_account::account`
//!
//! APIs which do not have tests:
//!
//! - Email APIs (`account/email.rs`) are not tested because email
//!   verification state and SMTP based sending depend on server email
//!   configuration which is not available in the test environment.
//! - Custom report APIs (`account/report.rs`) are not tested because
//!   custom reports are not configured in the test server config.
//! - Association membership APIs are not tested because the association
//!   feature is not enabled in the test server client features config.

pub(crate) mod ban;
pub(crate) mod client_features;
pub(crate) mod delete;
pub(crate) mod logout;
pub(crate) mod news;
pub(crate) mod notification;
pub(crate) mod register;
pub(crate) mod settings;
pub(crate) mod sign_in_with;
pub(crate) mod state;
pub(crate) mod verification;
