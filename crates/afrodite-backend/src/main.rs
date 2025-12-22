#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use simple_backend_config::file::SimpleBackendConfigFile;
use tls_client as _;

pub mod args;
pub mod build_info;
pub mod config_tools;

use std::process::ExitCode;

use build_info::{
    BUILD_INFO_CARGO_PKG_NAME, BUILD_INFO_CARGO_PKG_VERSION, BUILD_INFO_GIT_DESCRIBE,
};
use config::{
    args::{AppMode, ArgsConfig},
    get_config,
};
use server::{DatingAppServer, api_doc::ApiDoc};
use server_data::index::info::LocationIndexInfoCreator;
use test_mode::TestRunner;

fn main() -> ExitCode {
    tokio_rustls::rustls::crypto::ring::default_provider();

    let args = match args::get_config() {
        Ok(args) => args,
        Err(e) => return e,
    };

    handle_app_mode(args)
}

fn handle_app_mode(args: ArgsConfig) -> ExitCode {
    match args.mode {
        AppMode::Server(server_mode) => {
            let index_info = args.index_info;
            let config = get_config(
                server_mode,
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
            runtime.block_on(async { DatingAppServer::new(config).run().await });

            ExitCode::SUCCESS
        }
        AppMode::ManagerApi(api_client_mode) => {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let result = manager::client::handle_api_client_mode(api_client_mode).await;
                match result {
                    Ok(_) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("{e:?}");
                        ExitCode::FAILURE
                    }
                }
            })
        }
        AppMode::Manager => {
            let config = manager_config::get_config(
                BUILD_INFO_GIT_DESCRIBE.to_string(),
                BUILD_INFO_CARGO_PKG_VERSION.to_string(),
                BUILD_INFO_CARGO_PKG_NAME.to_string(),
            )
            .unwrap();
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async { manager::server::AppServer::new(config).run().await });
            ExitCode::SUCCESS
        }
        AppMode::ImageProcess(image_process) => {
            let config =
                SimpleBackendConfigFile::load(image_process.simple_backend_config).unwrap();
            match simple_backend_image_process::run_image_processing_loop(
                config.image_processing.unwrap_or_default(),
            ) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{e:?}");
                    ExitCode::FAILURE
                }
            }
        }
        AppMode::OpenApi => {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                println!("{}", ApiDoc::open_api_json_string().await.unwrap());
            });
            ExitCode::SUCCESS
        }
        AppMode::Config { mode } => {
            config_tools::handle_config_tools(mode).unwrap();
            ExitCode::SUCCESS
        }
        AppMode::RemoteBot(remote_bot_mode_config) => {
            let test_mode_config = remote_bot_mode_config.to_test_mode().unwrap();
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async { TestRunner::new(test_mode_config).run().await });
            ExitCode::SUCCESS
        }
        AppMode::Test(test_mode_config) => {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async { TestRunner::new(test_mode_config).run().await });
            ExitCode::SUCCESS
        }
    }
}
