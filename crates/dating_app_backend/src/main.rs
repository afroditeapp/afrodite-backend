#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod args;
pub mod build_info;

use std::process::ExitCode;

use build_info::{BUILD_INFO_CARGO_PKG_VERSION, BUILD_INFO_GIT_DESCRIBE};
use config::{args::AppMode, get_config};
use server::{api_doc::ApiDoc, DatingAppServer};
use server_data::index::LocationIndexInfoCreator;
use simple_backend_config::{args::ImageProcessModeArgs, file::ImageProcessingConfig};
use test_mode::TestRunner;

fn main() -> ExitCode {
    tokio_rustls::rustls::crypto::ring::default_provider();

    let args = match args::get_config() {
        Ok(args) => args,
        Err(e) => return e,
    };

    if let Some(AppMode::ImageProcess(settings)) = args.mode {
        let config = simple_backend_config::get_config(
            args.server,
        BUILD_INFO_GIT_DESCRIBE.to_string(),
        BUILD_INFO_CARGO_PKG_VERSION.to_string(),
        ).unwrap();
        return handle_image_process_mode(settings, config.image_processing());
    }

    if let Some(AppMode::OpenApi) = args.mode {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            println!("{}", ApiDoc::open_api_json_string().await.unwrap());
        });
        return ExitCode::SUCCESS;
    }
    let index_info = args.index_info;
    let config = get_config(
        args,
        BUILD_INFO_GIT_DESCRIBE.to_string(),
        BUILD_INFO_CARGO_PKG_VERSION.to_string(),
    )
    .unwrap();

    if index_info {
        println!(
            "{}",
            LocationIndexInfoCreator::new(config.location().clone())
                .create_all()
        );
        return ExitCode::SUCCESS;
    }

    let runtime = tokio::runtime::Runtime::new().unwrap();

    match config.current_mode() {
        Some(config::args::AppMode::ImageProcess(_)) | Some(config::args::AppMode::OpenApi) => {
            unreachable!()
        }
        Some(config::args::AppMode::Test(test_mode_config)) => {
            runtime.block_on(async { TestRunner::new(config, test_mode_config).run().await })
        }
        None => runtime.block_on(async { DatingAppServer::new(config).run().await }),
    }

    ExitCode::SUCCESS
}

fn handle_image_process_mode(
    args: ImageProcessModeArgs,
    config: ImageProcessingConfig,
) -> ExitCode {
    match simple_backend_image_process::handle_image(args, config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{:?}", e);
            ExitCode::FAILURE
        }
    }
}
