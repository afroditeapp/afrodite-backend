use config::{
    GetConfigError,
    args::{DataMode, DataModeSubMode, DataViewSubMode},
};
use database::{
    CurrentReadHandle, DatabaseHandleCreator, DbReaderRaw, current::read::GetDbReadCommandsCommon,
};
use database_media::current::read::GetDbReadCommandsMedia;
use simple_backend_config::{Database, args::ServerMode};

pub fn handle_data_tools(mode: DataMode) -> Result<(), GetConfigError> {
    if !mode.data_dir.exists() {
        eprintln!("Data directory '{:?}' not found", mode.data_dir);
        return Err(GetConfigError::SimpleBackendError);
    }
    if !mode.config_dir.exists() {
        eprintln!("Config directory '{:?}' not found", mode.config_dir);
        return Err(GetConfigError::SimpleBackendError);
    }

    let simple_config = simple_backend_config::get_config(
        ServerMode {
            data_dir: mode.data_dir,
            config_dir: mode.config_dir,
            ..Default::default()
        },
        String::new(),
        String::new(),
        false,
    )
    .map_err(|_| GetConfigError::SimpleBackendError)?;

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let (read, _close) = DatabaseHandleCreator::create_read_handle_from_config(
            &simple_config,
            &Database::Current,
        )
        .await
        .unwrap();

        let current_read = CurrentReadHandle(read);
        let reader = DbReaderRaw::new(&current_read);

        match mode.mode {
            DataModeSubMode::View { mode: view_mode } => match view_mode {
                DataViewSubMode::BotConfig => handle_view_bot_config(&reader).await,
                DataViewSubMode::ImageProcessingConfig => {
                    handle_view_image_processing_config(&reader).await
                }
            },
        }
    });

    Ok(())
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
