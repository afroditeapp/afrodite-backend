//! Admin custom email API tests
//!
//! APIs here send emails via SMTP when exercised with an admin account, so
//! only the permission check tests are done.

pub(crate) mod post_create_custom_email;
pub(crate) mod post_send_custom_email_draft_to_my_email_address;
pub(crate) mod post_update_custom_email;
