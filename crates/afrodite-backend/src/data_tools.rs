use std::{path::PathBuf, sync::Arc};

use config::{
    GetConfigError,
    args::{DataEditSubMode, DataLoadSubMode, DataMode, DataModeSubMode, DataViewSubMode},
};
use database::{
    DbReaderRaw, DbWriter,
    current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon},
};
use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use error_stack::{Result, report};
use model::{
    AccountId, AccountIdInternal, BotConfig, DynamicServerConfig, EmailMessages,
    ImageProcessingDynamicConfig,
};
use model_server_data::ProfileAttributesSchemaExport;
use server_data::{
    db_manager::{DatabaseManager, InternalWriting, RouterDatabaseWriteHandle},
    profile_attributes::load_profile_attributes_from_db,
    write::GetWriteCommandsCommon,
};
use simple_backend_config::args::ServerMode;
use simple_backend_utils::dir::abs_path_for_directory_or_file_which_might_not_exists;

use crate::process_lock;

mod csv;

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
            DataLoadSubMode::ProfileAttributes { file } => {
                *file = abs_path_for_directory_or_file_which_might_not_exists(&*file)
                    .map_err(|_| report!(GetConfigError::GetWorkingDir))?;
            }
            DataLoadSubMode::DynamicClientFeatures { file } => {
                *file = abs_path_for_directory_or_file_which_might_not_exists(&*file)
                    .map_err(|_| report!(GetConfigError::GetWorkingDir))?;
            }
            DataLoadSubMode::DynamicServerConfig { file } => {
                *file = abs_path_for_directory_or_file_which_might_not_exists(&*file)
                    .map_err(|_| report!(GetConfigError::GetWorkingDir))?;
            }
            DataLoadSubMode::ProfileAttributeValuesCsv { csv_file, .. } => {
                *csv_file = abs_path_for_directory_or_file_which_might_not_exists(&*csv_file)
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

    let _lock = if matches!(
        mode.mode,
        DataModeSubMode::Load { .. } | DataModeSubMode::Edit { .. }
    ) {
        let lock = process_lock::acquire_server_lock(&mode.data_dir)
            .map_err(|e| report!(GetConfigError::LoadFileError).attach_printable(e))?;
        Some(lock)
    } else {
        None
    };

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
                DataViewSubMode::ProfileAttributes => handle_view_profile_attributes(&reader).await,
                DataViewSubMode::DynamicClientFeatures => {
                    handle_view_dynamic_client_features(&reader).await
                }
                DataViewSubMode::DynamicServerConfig => {
                    handle_view_dynamic_server_config(&reader).await
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
                    DataLoadSubMode::ProfileAttributes { file } => {
                        handle_load_profile_attributes(&writer, file).await
                    }
                    DataLoadSubMode::DynamicClientFeatures { file } => {
                        handle_load_dynamic_client_features(&write_handle, file).await
                    }
                    DataLoadSubMode::DynamicServerConfig { file } => {
                        handle_load_dynamic_server_config(&write_handle, file).await
                    }
                    DataLoadSubMode::ProfileAttributeValuesCsv {
                        attribute_id,
                        csv_file,
                        delimiter,
                        values_column_index,
                        group_values_column_index,
                        start_row_index,
                        translations,
                    } => {
                        csv::handle_load_profile_attributes_values_from_csv(
                            &reader,
                            &writer,
                            attribute_id,
                            csv_file,
                            delimiter,
                            values_column_index,
                            group_values_column_index,
                            start_row_index,
                            translations,
                        )
                        .await
                    }
                }
            }
            DataModeSubMode::Edit { mode: edit_mode } => {
                let writer = DbWriter::new(write_handle.current_write_handle());

                match edit_mode {
                    DataEditSubMode::GrantAdminEditPermissions { account_id } => {
                        handle_grant_admin_edit_permissions(&reader, &writer, account_id).await
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

async fn handle_load_profile_attributes(writer: &DbWriter<'_>, file: PathBuf) {
    let content = std::fs::read_to_string(&file)
        .unwrap_or_else(|e| panic!("Failed to read file {:?}: {}", file, e));

    let file_content: ProfileAttributesSchemaExport =
        toml::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse TOML: {}", e));

    let profile_attrs = file_content
        .validate()
        .unwrap_or_else(|e| panic!("Validation failed: {}", e));

    let attr_count = profile_attrs.attributes().len();

    writer
        .db_transaction_raw(move |mut cmds| {
            cmds.common()
                .profile_attributes()
                .delete_all_profile_attributes()?;
            for attr in profile_attrs.attributes() {
                cmds.common()
                    .profile_attributes()
                    .upsert_profile_attribute(attr.attribute())?;
            }
            cmds.common()
                .profile_attributes()
                .upsert_profile_attributes_order_mode(profile_attrs.attribute_order())?;

            cmds.common()
                .client_config()
                .increment_client_config_sync_version_for_every_account()?;

            Ok(())
        })
        .await
        .unwrap();

    println!(
        "Successfully loaded {} profile attributes into database",
        attr_count
    );
}

async fn handle_view_profile_attributes(reader: &DbReaderRaw<'_>) {
    let manager = load_profile_attributes_from_db(reader).await.unwrap();

    let export = manager.read().await.export();

    println!("{}", toml::to_string_pretty(&export).unwrap());
}

async fn handle_load_dynamic_client_features(
    write_handle: &RouterDatabaseWriteHandle,
    file: PathBuf,
) {
    let content = std::fs::read_to_string(&file)
        .unwrap_or_else(|e| panic!("Failed to read file {:?}: {}", file, e));

    let config: model::DynamicClientFeaturesConfig =
        toml::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse TOML: {}", e));

    write_handle
        .common()
        .client_config()
        .upsert_dynamic_client_features_config(config)
        .await
        .unwrap();

    println!("Successfully loaded dynamic client features config into database");
}

async fn handle_view_dynamic_client_features(reader: &DbReaderRaw<'_>) {
    let config = reader
        .db_read(|mut mode| {
            Ok(mode
                .common()
                .client_config()
                .dynamic_client_features()?
                .map(|(_, config)| config)
                .unwrap_or_default())
        })
        .await
        .unwrap();

    println!("{}", toml::to_string_pretty(&config).unwrap());
}

async fn handle_load_dynamic_server_config(
    write_handle: &RouterDatabaseWriteHandle,
    file: PathBuf,
) {
    let content = std::fs::read_to_string(&file)
        .unwrap_or_else(|e| panic!("Failed to read file {:?}: {}", file, e));

    let config: DynamicServerConfig =
        toml::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse TOML: {}", e));

    write_handle
        .common()
        .client_config()
        .upsert_dynamic_server_config(config)
        .await
        .unwrap();

    println!("Successfully loaded dynamic server config into database");
}

async fn handle_view_dynamic_server_config(reader: &DbReaderRaw<'_>) {
    let config = reader
        .db_read(|mut mode| {
            Ok(mode
                .common()
                .client_config()
                .dynamic_server_config()?
                .unwrap_or_default())
        })
        .await
        .unwrap();

    println!("{}", toml::to_string_pretty(&config).unwrap());
}

async fn handle_grant_admin_edit_permissions(
    reader: &DbReaderRaw<'_>,
    writer: &DbWriter<'_>,
    account_id: AccountId,
) {
    let internal_id = reader
        .db_read(move |mut cmds| {
            let internal_id = cmds
                .common()
                .account_ids_internal()?
                .into_iter()
                .find(|id| id.as_id() == account_id)
                .ok_or_else(|| report!(database::DieselDatabaseError::NotFound))?;

            Ok(internal_id)
        })
        .await
        .unwrap();

    writer
        .db_transaction_raw(move |mut cmds| {
            let account = cmds.read().common().account(internal_id)?;
            let new_permissions = model::Permissions {
                admin_edit_permissions: true,
                ..account.permissions()
            };

            let _updated_account = cmds.common().state().update_syncable_account_data(
                internal_id,
                account,
                move |account| {
                    account.permissions = new_permissions;
                    Ok(())
                },
            )?;

            Ok(())
        })
        .await
        .unwrap();

    println!(
        "Granted Permissions::admin_edit_permissions for account {}",
        internal_id.as_id()
    );
}
