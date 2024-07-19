#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod args;
pub mod build_info;

use std::process::ExitCode;

use build_info::{BUILD_INFO_CARGO_PKG_VERSION, BUILD_INFO_GIT_DESCRIBE};
use config::{args::AppMode, get_config};
use server::{api_doc::ApiDoc, PihkaServer};
use server_data::index::LocationIndexInfoCreator;
use simple_backend_config::args::ImageProcessModeArgs;
use test_mode::TestRunner;

fn main() -> ExitCode {
    let args = match args::get_config() {
        Ok(args) => args,
        Err(e) => return e,
    };

    if let Some(AppMode::ImageProcess(settings)) = args.mode {
        return handle_image_process_mode(settings);
    }

    if let Some(AppMode::OpenApi) = args.mode {
        println!("{}", ApiDoc::open_api_json_string().unwrap());
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
        println!("{}", LocationIndexInfoCreator::new(config.into()).create_all());
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
        None => runtime.block_on(async { PihkaServer::new(config).run().await }),
    }

    ExitCode::SUCCESS
}

fn handle_image_process_mode(settings: ImageProcessModeArgs) -> ExitCode {
    let settings = simple_backend_image_process::Settings {
        input: settings.input,
        input_file_type: settings.input_file_type,
        output: settings.output,
        quality: settings.quality as f32,
    };

    match simple_backend_image_process::handle_image(settings) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{:?}", e);
            ExitCode::FAILURE
        }
    }
}
