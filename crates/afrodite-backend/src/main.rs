#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use tls_client as _;

pub mod args;
pub mod build_info;
pub mod config_tools;

use std::process::ExitCode;

use build_info::{
    BUILD_INFO_CARGO_PKG_NAME, BUILD_INFO_CARGO_PKG_VERSION, BUILD_INFO_GIT_DESCRIBE,
};
use config::{args::AppMode, get_config};
use manager_config::args::ManagerApiClientMode;
use server::{DatingAppServer, api_doc::ApiDoc};
use server_data::index::info::LocationIndexInfoCreator;
use simple_backend_config::file::ImageProcessingConfig;
use test_mode::TestRunner;

fn main() -> ExitCode {
    tokio_rustls::rustls::crypto::ring::default_provider();

    let args = match args::get_config() {
        Ok(args) => args,
        Err(e) => return e,
    };

    if let Some(AppMode::ManagerApi(api_client_mode)) = args.mode {
        return handle_manager_api_client_mode(api_client_mode);
    }

    if let Some(AppMode::Manager) = args.mode {
        let config = manager_config::get_config(
            BUILD_INFO_GIT_DESCRIBE.to_string(),
            BUILD_INFO_CARGO_PKG_VERSION.to_string(),
            BUILD_INFO_CARGO_PKG_NAME.to_string(),
        )
        .unwrap();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async { manager::server::AppServer::new(config).run().await });
        return ExitCode::SUCCESS;
    }

    if let Some(AppMode::ImageProcess) = args.mode {
        let config = simple_backend_config::get_config(
            args.server,
            BUILD_INFO_GIT_DESCRIBE.to_string(),
            BUILD_INFO_CARGO_PKG_VERSION.to_string(),
            true,
        )
        .unwrap();
        return handle_image_process_mode(config.image_processing());
    }

    if let Some(AppMode::OpenApi) = args.mode {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            println!("{}", ApiDoc::open_api_json_string().await.unwrap());
        });
        return ExitCode::SUCCESS;
    }

    if let Some(AppMode::Config { mode }) = args.mode {
        config_tools::handle_config_tools(mode).unwrap();
        return ExitCode::SUCCESS;
    }

    let index_info = args.index_info;
    let config = get_config(
        args,
        BUILD_INFO_GIT_DESCRIBE.to_string(),
        BUILD_INFO_CARGO_PKG_VERSION.to_string(),
        true,
    )
    .unwrap();

    if index_info {
        println!(
            "{}",
            LocationIndexInfoCreator::new(config.location().clone()).create_all()
        );
        return ExitCode::SUCCESS;
    }

    let runtime = tokio::runtime::Runtime::new().unwrap();

    match config.current_mode() {
        Some(config::args::AppMode::Config { .. })
        | Some(config::args::AppMode::Manager)
        | Some(config::args::AppMode::ManagerApi(_))
        | Some(config::args::AppMode::ImageProcess)
        | Some(config::args::AppMode::OpenApi) => {
            unreachable!()
        }
        Some(config::args::AppMode::RemoteBot(test_mode_config)) => {
            let test_mode_config = test_mode_config.to_test_mode().unwrap();
            runtime.block_on(async { TestRunner::new(config, test_mode_config).run().await })
        }
        Some(config::args::AppMode::Test(test_mode_config)) => {
            runtime.block_on(async { TestRunner::new(config, test_mode_config).run().await })
        }
        None => runtime.block_on(async { DatingAppServer::new(config).run().await }),
    }

    ExitCode::SUCCESS
}

fn handle_manager_api_client_mode(args: ManagerApiClientMode) -> ExitCode {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let result = manager::client::handle_api_client_mode(args).await;
        match result {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e:?}");
                ExitCode::FAILURE
            }
        }
    })
}

fn handle_image_process_mode(config: ImageProcessingConfig) -> ExitCode {
    match simple_backend_image_process::run_image_processing_loop(config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e:?}");
            ExitCode::FAILURE
        }
    }
}
