//! API constants for server and test_mode crates.
//! These are defined here to make parallel
//! compilation possible.

use obfuscate_api_macro::obfuscate_api;

#[obfuscate_api]
pub const PATH_CONNECT: &str = "/common_api/connect";

pub const ACCESS_TOKEN_HEADER_STR: &str = "x-access-token";
