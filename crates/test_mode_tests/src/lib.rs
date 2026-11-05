#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! Tests for `common_api` and `common_admin_api` routes in the `server_api`
//! crate.
//!
//! Test modules mirror the server route module layout:
//!
//! * `common/<module>/<api_name>.rs` - tests for `server_api::common` routes,
//!   e.g. tests for `crates/server_api/src/common/data_export.rs` are in
//!   `common/data_export/<api_name>.rs` where `<api_name>` is the server
//!   side handler function name
//! * `common_admin/<module>/<api_name>.rs` - tests for
//!   `server_api::common_admin` routes
//!
//! All `server_api::common` routes which require an access token have tests.
//! Public routes (`server_online`,
//! `manual_server_maintenance_info_for_another_server`, static file serving,
//! WebSocket connect) do not have authorization tests because they do not
//! require an access token. WebSocket connecting is covered indirectly by
//! `Account::register_and_login`.
//!
//! Admin routes are tested with `request_without_admin_permission_fails`
//! test using an account without admin permissions, together with a
//! successful admin account test.
//!
//! `common_admin` APIs without a dedicated test file (report APIs are used
//! indirectly via admin account in `original/report.rs`):
//!
//! - `config`
//!   - get_bot_config
//!   - post_bot_config
//!   - get_dynamic_server_config
//!   - post_dynamic_server_config
//! - `maintenance`
//!   - get_server_version
//!   - get_maintenance_notification
//!   - post_edit_maintenance_notification
//! - `manager`
//!   - get_manager_instance_names
//!   - get_system_info
//!   - get_software_update_status
//!   - post_trigger_software_update_download
//!   - post_trigger_software_update_install
//!   - post_trigger_server_data_reset
//!   - post_trigger_server_restart
//!   - post_trigger_server_shutdown
//!   - post_trigger_system_reboot
//!   - post_trigger_system_shutdown
//!   - get_scheduled_tasks_status
//!   - post_schedule_task
//!   - post_unschedule_task
//! - `report`
//!   - post_get_report_queue_page
//!   - post_process_reports
//!   - get_latest_report_iterator_start_position
//!   - post_get_report_iterator_page
//!   - post_get_chat_message_reports
//! - `statistics`
//!   - post_get_perf_data
//!   - post_get_api_usage_data
//!   - post_get_ip_address_usage_data
//!   - post_get_ip_country_statistics

mod common;
mod common_admin;
mod original;

pub fn call_this_to_make_sure_that_crate_is_linked() {}
