//! Tests which existed before the API authorization test suite was
//! generated. Kept unmodified in their original form.
//!
//! `admin.rs` and `initial_setup.rs` used to live at the crate root
//! (`crates/test_mode_tests_account/src/`).
//!
//! Admin APIs exercised indirectly by these tests, so those APIs do not
//! need a dedicated test file:
//!
//! - none (admin tests exercise the same `account_api` routes as the
//!   generated suite)

mod admin;
mod initial_setup;
