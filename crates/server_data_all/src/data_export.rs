use std::{io::Write, sync::Arc};

use config::Config;
use database::{DbReadMode, DieselDatabaseError};
use error_stack::ResultExt;
use model::{ContentId, DataExportType};
use serde::Serialize;
use server_data::{
    DataError,
    app::GetConfig,
    data_export::DataExportCmd,
    file::{FileWrite, utils::FileDir},
    read::DbRead,
    result::Result,
    write_commands::WriteCommandRunnerHandle,
};
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::data_export::{
    admin::generate_admin_data_export_json, user::generate_user_data_export_json,
};

mod admin;
mod user;

pub async fn data_export(
    write_handle: &WriteCommandRunnerHandle,
    zip_main_directory_name: String,
    cmd: DataExportCmd,
) -> Result<(), DataError> {
    write_handle
        .write(move |cmds| async move {
            let archive = cmds.files().tmp_dir(cmd.target().0.into()).data_export();
            archive.overwrite_and_remove_if_exists().await?;

            let config = cmds.config_arc();
            let file_dir = cmds.files().clone();
            cmds.db_read(move |cmds| {
                db_data_export(config, cmds, file_dir, zip_main_directory_name, cmd)?;
                Ok(())
            })
            .await?;
            Ok(())
        })
        .await?;

    Ok(())
}

fn db_data_export(
    config: Arc<Config>,
    mut current: DbReadMode,
    file_dir: FileDir,
    zip_main_directory_name: String,
    cmd: DataExportCmd,
) -> error_stack::Result<(), DieselDatabaseError> {
    let archive = file_dir.tmp_dir(cmd.target().0.into()).data_export();

    let file =
        std::fs::File::create_new(archive.path()).change_context(DieselDatabaseError::File)?;
    let mut zip_writer = zip::ZipWriter::new(file);

    let data = generate_user_data_export_json(&mut current, cmd.source())?;
    write_json_file(&data, "user", &zip_main_directory_name, &mut zip_writer)?;

    if cmd.data_export_type() == DataExportType::Admin {
        let data = generate_admin_data_export_json(&config, &mut current, cmd.source())?;
        write_json_file(&data, "admin", &zip_main_directory_name, &mut zip_writer)?;
    }

    for c in data.media.content {
        let data = file_dir
            .media_content(cmd.source().0.uuid, c.cid)
            .read_all_blocking()
            .change_context(DieselDatabaseError::File)?;
        write_media_content(c.cid, &data, &zip_main_directory_name, &mut zip_writer)?;
    }

    let mut file = zip_writer
        .finish()
        .change_context(DieselDatabaseError::Zip)?;
    file.flush().change_context(DieselDatabaseError::File)?;

    Ok(())
}

fn write_json_file<T: Serialize>(
    json: &T,
    json_name: &str,
    zip_main_directory_name: &str,
    zip_writer: &mut ZipWriter<std::fs::File>,
) -> error_stack::Result<(), DieselDatabaseError> {
    let json =
        serde_json::to_string_pretty(&json).change_context(DieselDatabaseError::SerdeSerialize)?;
    let file_name = format!("{zip_main_directory_name}/{json_name}.json");
    zip_writer
        .start_file(file_name, SimpleFileOptions::default())
        .change_context(DieselDatabaseError::Zip)?;
    zip_writer
        .write_all(json.as_bytes())
        .change_context(DieselDatabaseError::Zip)?;
    Ok(())
}

fn write_media_content(
    content_id: ContentId,
    data: &[u8],
    zip_main_directory_name: &str,
    zip_writer: &mut ZipWriter<std::fs::File>,
) -> error_stack::Result<(), DieselDatabaseError> {
    let content_file_name = content_id.content_file_name();
    let file_name = format!("{zip_main_directory_name}/media/{content_file_name}.jpg");
    zip_writer
        .start_file(file_name, SimpleFileOptions::default())
        .change_context(DieselDatabaseError::Zip)?;
    zip_writer
        .write_all(data)
        .change_context(DieselDatabaseError::Zip)?;
    Ok(())
}
