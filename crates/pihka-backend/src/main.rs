pub mod build_info;

use build_info::{BUILD_INFO_GIT_DESCRIBE, BUILD_INFO_CARGO_PKG_VERSION};
use server::PihkaServer;
use test_mode::TestRunner;
use config::get_config;


fn main() {
    // TODO: print commit ID to logs if build directory was clean
    let config =
        get_config(
            build_info::build_info,
            BUILD_INFO_GIT_DESCRIBE.to_string(),
            BUILD_INFO_CARGO_PKG_VERSION.to_string(),
        ).unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();

    if let Some(test_mode_config) = config.test_mode() {
        runtime.block_on(async { TestRunner::new(config, test_mode_config).run().await })
    } else {
        runtime.block_on(async { PihkaServer::new(config).run().await })
    }
}
