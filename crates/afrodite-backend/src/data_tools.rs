use std::{path::PathBuf, sync::Arc};

use config::{
    GetConfigError,
    args::{DataLoadSubMode, DataMode, DataModeSubMode, DataViewSubMode},
};
use database::{
    DbReaderRaw, DbWriter,
    current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon},
};
use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use error_stack::{Result, report};
use model::{AccountIdInternal, BotConfig, EmailMessages, ImageProcessingDynamicConfig};
use server_data::db_manager::{DatabaseManager, InternalWriting};
use simple_backend_config::args::ServerMode;
use simple_backend_utils::dir::abs_path_for_directory_or_file_which_might_not_exists;

pub fn handle_data_tools(mut mode: DataMode) -> Result<(), GetConfigError> {
    // Convert config file paths to absolute paths because get_config
    // changes working directory.
    if let DataModeSubMode::Load { mode: load_mode } = &mut mode.mode {
        match load_mode {
            DataLoadSubMode::BotConfig { file } => {
                *file = abs_path_for_directory_or_file_which_might_not_exists(&*file)
                    .map_err(|_| report!(GetConfigError::GetWorkingDir))?;
            }
            DataLoadSubMode::ImageProcessingConfig { file } => {
                *file = abs_path_for_directory_or_file_which_might_not_exists(&*file)
                    .map_err(|_| report!(GetConfigError::GetWorkingDir))?;
            }
        }
    }

    if !mode.data_dir.exists() {
        eprintln!("Data directory '{:?}' not found", mode.data_dir);
        return Err(report!(GetConfigError::SimpleBackendError));
    }
    if !mode.config_dir.exists() {
        eprintln!("Config directory '{:?}' not found", mode.config_dir);
        return Err(report!(GetConfigError::SimpleBackendError));
    }

    let config = config::get_config(
        ServerMode {
            data_dir: mode.data_dir,
            config_dir: mode.config_dir,
            ..Default::default()
        },
        String::new(),
        String::new(),
        false,
    )?;

    let config = Arc::new(config);

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let (push_notification_sender, _push_notification_receiver) =
            server_common::push_notifications::channel();
        let (email_sender, _email_receiver) =
            simple_backend::email::channel::<AccountIdInternal, EmailMessages>();

        let (db_manager, read_handle, write_handle) =
            DatabaseManager::new(config.clone(), push_notification_sender, email_sender)
                .await
                .unwrap();

        let reader = DbReaderRaw::new(read_handle.read_handle_raw());

        match mode.mode {
            DataModeSubMode::View { mode: view_mode } => match view_mode {
                DataViewSubMode::BotConfig => handle_view_bot_config(&reader).await,
                DataViewSubMode::ImageProcessingConfig => {
                    handle_view_image_processing_config(&reader).await
                }
            },
            DataModeSubMode::Load { mode: load_mode } => {
                let writer = DbWriter::new(write_handle.current_write_handle());

                match load_mode {
                    DataLoadSubMode::BotConfig { file } => {
                        handle_load_bot_config(&writer, file).await
                    }
                    DataLoadSubMode::ImageProcessingConfig { file } => {
                        handle_load_image_processing_config(&writer, file).await
                    }
                }
            }
        }

        db_manager.close().await;
    });

    Ok(())
}

async fn handle_load_bot_config(writer: &DbWriter<'_>, file: PathBuf) {
    let content = std::fs::read_to_string(file).unwrap();
    let config: BotConfig = toml::from_str(&content).unwrap();

    writer
        .db_transaction_raw(move |mut cmds| {
            cmds.common().bot_config().upsert_bot_config(&config)?;
            Ok(())
        })
        .await
        .unwrap();
}

async fn handle_load_image_processing_config(writer: &DbWriter<'_>, file: PathBuf) {
    let content = std::fs::read_to_string(file).unwrap();
    let config: ImageProcessingDynamicConfig = toml::from_str(&content).unwrap();

    writer
        .db_transaction_raw(move |mut cmds| {
            cmds.media_admin()
                .image_processing_config()
                .upsert_image_processing_config(&config)?;
            Ok(())
        })
        .await
        .unwrap();
}

async fn handle_view_bot_config(reader: &DbReaderRaw<'_>) {
    let config = reader
        .db_read(|mut mode| Ok(mode.common().bot_config().bot_config()?.unwrap_or_default()))
        .await
        .unwrap();

    println!("{}", toml::to_string_pretty(&config).unwrap());
}

async fn handle_view_image_processing_config(reader: &DbReaderRaw<'_>) {
    let config = reader
        .db_read(|mut mode| {
            Ok(mode
                .media_admin()
                .image_processing_config()
                .image_processing_config()?
                .unwrap_or_default())
        })
        .await
        .unwrap();

    println!("{}", toml::to_string_pretty(&config).unwrap());
}
