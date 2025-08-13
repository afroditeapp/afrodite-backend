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
    admin::common::AdminDataExportJsonCommon,
    user::{
        account::UserDataExportJsonAccount, common::UserDataExportJsonCommon,
        media::UserDataExportJsonMedia, profile::UserDataExportJsonProfile,
    },
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
    let mut writer = DataExportArchiveWriter {
        zip_main_directory_name,
        zip_writer: zip::ZipWriter::new(file),
    };

    writer.write_user_json_file(
        "common",
        &UserDataExportJsonCommon::query(&mut current, cmd.source())?,
    )?;
    writer.write_user_json_file(
        "profile",
        &UserDataExportJsonProfile::query(&mut current, cmd.source())?,
    )?;
    writer.write_user_json_file(
        "account",
        &UserDataExportJsonAccount::query(&mut current, cmd.source())?,
    )?;
    let media = UserDataExportJsonMedia::query(&mut current, cmd.source())?;
    writer.write_user_json_file("media", &media)?;

    if cmd.data_export_type() == DataExportType::Admin {
        writer.write_admin_json_file(
            "common",
            &AdminDataExportJsonCommon::query(&config, &mut current, cmd.source())?,
        )?;
    }

    for c in media.content {
        let data = file_dir
            .media_content(cmd.source().0.uuid, c.cid)
            .read_all_blocking()
            .change_context(DieselDatabaseError::File)?;
        writer.write_media_content(c.cid, &data)?;
    }

    let mut file = writer
        .zip_writer
        .finish()
        .change_context(DieselDatabaseError::Zip)?;
    file.flush().change_context(DieselDatabaseError::File)?;

    Ok(())
}

struct DataExportArchiveWriter {
    zip_main_directory_name: String,
    zip_writer: ZipWriter<std::fs::File>,
}

impl DataExportArchiveWriter {
    fn write_json_file_internal<T: Serialize>(
        &mut self,
        dir_name: &str,
        json_name: &str,
        json: &T,
    ) -> error_stack::Result<(), DieselDatabaseError> {
        let zip_main_directory_name = &self.zip_main_directory_name;
        let file_name = format!("{zip_main_directory_name}/{dir_name}/{json_name}.json");
        let json = serde_json::to_string_pretty(&json)
            .change_context(DieselDatabaseError::SerdeSerialize)?;
        self.zip_writer
            .start_file(file_name, SimpleFileOptions::default())
            .change_context(DieselDatabaseError::Zip)?;
        self.zip_writer
            .write_all(json.as_bytes())
            .change_context(DieselDatabaseError::Zip)?;
        Ok(())
    }

    fn write_user_json_file<T: Serialize>(
        &mut self,
        json_name: &str,
        json: &T,
    ) -> error_stack::Result<(), DieselDatabaseError> {
        self.write_json_file_internal("user", json_name, json)
    }

    fn write_admin_json_file<T: Serialize>(
        &mut self,
        json_name: &str,
        json: &T,
    ) -> error_stack::Result<(), DieselDatabaseError> {
        self.write_json_file_internal("admin", json_name, json)
    }

    fn write_media_content(
        &mut self,
        content_id: ContentId,
        data: &[u8],
    ) -> error_stack::Result<(), DieselDatabaseError> {
        let zip_main_directory_name = &self.zip_main_directory_name;
        let content_file_name = content_id.content_file_name();
        let file_name = format!("{zip_main_directory_name}/media/{content_file_name}.jpg");
        self.zip_writer
            .start_file(file_name, SimpleFileOptions::default())
            .change_context(DieselDatabaseError::Zip)?;
        self.zip_writer
            .write_all(data)
            .change_context(DieselDatabaseError::Zip)?;
        Ok(())
    }
}
