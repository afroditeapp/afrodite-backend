//! Build info
//!

pub const BUILD_INFO_CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const BUILD_INFO_GIT_COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");
pub const BUILD_INFO_GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
pub const BUILD_INFO_GIT_BRANCH: &str = env!("VERGEN_GIT_BRANCH");
pub const BUILD_INFO_RUSTC_SEMVER: &str = env!("VERGEN_RUSTC_SEMVER");
pub const BUILD_INFO_RUSTC_HOST_TRIPLE: &str = env!("VERGEN_RUSTC_HOST_TRIPLE");
pub const BUILD_INFO_CARGO_TARGET_TRIPLE: &str = env!("VERGEN_CARGO_TARGET_TRIPLE");
pub const BUILD_INFO_CARGO_DEBUG: &str = env!("VERGEN_CARGO_DEBUG");
pub const BUILD_INFO_CARGO_FEATURES: &str = env!("VERGEN_CARGO_FEATURES");
pub const BUILD_INFO_CARGO_OPT_LEVEL: &str = env!("VERGEN_CARGO_OPT_LEVEL");


pub fn build_info() -> String {
    format!(
        "git commit hash: {}\ngit describe: {}\ngit branch: {}\nrustc semver: {}\nrustc host triple: {}\ncargo target triple: {}\ncargo debug: {}\ncargo features: {}\ncargo opt level: {}",
        BUILD_INFO_GIT_COMMIT_HASH,
        BUILD_INFO_GIT_DESCRIBE,
        BUILD_INFO_GIT_BRANCH,
        BUILD_INFO_RUSTC_SEMVER,
        BUILD_INFO_RUSTC_HOST_TRIPLE,
        BUILD_INFO_CARGO_TARGET_TRIPLE,
        BUILD_INFO_CARGO_DEBUG,
        BUILD_INFO_CARGO_FEATURES,
        BUILD_INFO_CARGO_OPT_LEVEL,
    )
}
