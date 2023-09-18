#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod build_info;
pub mod args;

use std::process::exit;

use build_info::{BUILD_INFO_CARGO_PKG_VERSION, BUILD_INFO_GIT_DESCRIBE};
use config::{get_config, args::{AppMode, ImageProcessMode}};
use server::PihkaServer;
use test_mode::TestRunner;

fn main() {
    let args = args::get_config();

    if let Some(AppMode::ImageProcess(settings)) = args.mode {
        handle_image_process_mode(settings);
        return;
    }

    let config = get_config(
        args,
        BUILD_INFO_GIT_DESCRIBE.to_string(),
        BUILD_INFO_CARGO_PKG_VERSION.to_string(),
    )
    .unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();

    match config.current_mode() {
        Some(config::args::AppMode::ImageProcess(_)) => unreachable!(),
        Some(config::args::AppMode::Test(test_mode_config)) =>
            runtime.block_on(async {
                TestRunner::new(config, test_mode_config).run().await
            }),
        None =>
            runtime.block_on(async {
                PihkaServer::new(config).run().await
            }),
    }
}


fn handle_image_process_mode(settings: ImageProcessMode) {
    let settings = image_process::Settings {
        input: settings.input,
        output: settings.output,
        quality: settings.quality as f32,
    };

    match image_process::handle_image(settings) {
        Ok(()) => exit(0),
        Err(e) => {
            eprintln!("{:?}", e);
            exit(1);
        }
    }
}