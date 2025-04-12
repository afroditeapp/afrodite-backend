

use std::num::Wrapping;
use std::sync::atomic::{AtomicU32, Ordering};

use manager_api::backup::BackupSourceClient;
use manager_model::{AccountAndContent, Sha256Bytes, SourceToTargetMessage, TargetToSourceMessage};
use model::{AccountId, ContentId};
use server_api::app::GetConfig;
use server_api::DataError;
use server_api::{
    app::ReadData,
    result::WrappedContextExt,
};
use server_common::result::{Result, WrappedResultExt};
use server_data::read::GetReadCommandsCommon;
use server_data_media::read::GetReadMediaCommands;
use server_state::S;
use sha2::{Digest, Sha256};
use simple_backend::ServerQuitWatcher;
use simple_backend::app::GetManagerApi;
use simple_backend_config::SqliteDatabase;
use simple_backend_utils::file::overwrite_and_remove_if_exists;
use tokio::io::AsyncReadExt;
use tokio::sync::broadcast::error::TryRecvError;

use super::ScheduledTaskError;

const DATABASE_BACKUP_TMP_FILE_NAME: &str = "database_backup.tmp";

static BACKUP_SESSION: AtomicU32 = AtomicU32::new(0);

pub async fn backup_data(
    state: &S,
    quit_notification: &mut ServerQuitWatcher,
) -> Result<(), ScheduledTaskError> {
    let Some(mut backup_client) = state
        .manager_api_client()
        .new_backup_connection(BACKUP_SESSION.fetch_add(1, Ordering::Relaxed))
        .await
        .change_context(ScheduledTaskError::Backup)? else {
            // Backup link password is not configured
            return Ok(());
        };

    backup_client.send_message(SourceToTargetMessage::StartBackupSession)
        .await
        .change_context(ScheduledTaskError::Backup)?;

    let accounts = state
        .read()
        .common()
        .account_ids_internal_vec()
        .await
        .change_context(ScheduledTaskError::DatabaseError)?;

    for ids in accounts.chunks(100) {
        if quit_notification.try_recv() != Err(TryRecvError::Empty) {
            return Err(ScheduledTaskError::QuitRequested.report());
        }

        let mut data = vec![];

        for &a in ids {
            let content = state
                .read()
                .media()
                .all_account_media_content(a)
                .await
                .change_context(ScheduledTaskError::DatabaseError)?;

            let content_ids = content.iter().map(|v| v.content_id().cid).collect();

            data.push(AccountAndContent {
                account_id: a.as_id().aid,
                content_ids,
            });
        }

        backup_client.send_message(SourceToTargetMessage::ContentList { data })
            .await
            .change_context(ScheduledTaskError::Backup)?;

        loop {
            let m = backup_client.receive_message()
                .await
                .change_context(ScheduledTaskError::Backup)?;

            match m {
                TargetToSourceMessage::ContentListSyncDone => break,
                TargetToSourceMessage::ContentQuery { account_id, content_id } => {
                    let content_data = state
                        .read()
                        .media()
                        .content_data(AccountId { aid: account_id }, ContentId { cid: content_id })
                        .await
                        .change_context(ScheduledTaskError::DatabaseError)?;
                    let data = content_data
                        .read_all()
                        .await
                        .change_context(ScheduledTaskError::FileReadingError)?;
                    let mut hasher = Sha256::new();
                    hasher.update(&data);
                    let result = hasher.finalize();
                    backup_client.send_message(SourceToTargetMessage::ContentQueryAnswer { sha256: Sha256Bytes(result.into()), data })
                        .await
                        .change_context(ScheduledTaskError::Backup)?;
                }
            }
        }
    }

    // Empty file name ends content backup waiting
    backup_client.send_message(SourceToTargetMessage::ContentList { data: vec![] })
        .await
        .change_context(ScheduledTaskError::Backup)?;

    let tmp_db = tmp_db_path_string(state)?;

    let databases = state.config().simple_backend().databases();

    handle_db(
        &mut backup_client,
        &tmp_db,
        &databases.current,
        state
            .read()
            .common()
            .backup_current_database(tmp_db.clone()),
    ).await?;
    handle_db(
        &mut backup_client,
        &tmp_db,
        &databases.history,
        state
            .read()
            .common_history()
            .backup_history_database(tmp_db.clone()),
    ).await?;

    // Empty file name ends file backup waiting
    backup_client.send_message(SourceToTargetMessage::StartFileBackup { sha256: Sha256Bytes([0; 32]), file_name: String::new() })
        .await
        .change_context(ScheduledTaskError::Backup)?;

    Ok(())
}

async fn handle_db(
    backup_client: &mut BackupSourceClient,
    tmp_db: &str,
    db_name: &SqliteDatabase,
    create_backup_file: impl Future<Output=Result<(), DataError>>,
) -> Result<(), ScheduledTaskError> {
    overwrite_and_remove_if_exists(tmp_db)
        .await
        .change_context(ScheduledTaskError::Backup)?;

    create_backup_file
        .await
        .change_context(ScheduledTaskError::DatabaseError)?;

    send_backup_db(db_name, tmp_db, backup_client).await?;

    overwrite_and_remove_if_exists(tmp_db)
        .await
        .change_context(ScheduledTaskError::Backup)?;

    Ok(())
}

async fn send_backup_db(
    info: &SqliteDatabase,
    tmp_db_path: &str,
    backup_client: &mut BackupSourceClient,
) -> Result<(), ScheduledTaskError> {
    let sha256 = calculate_hash(tmp_db_path).await?;
    backup_client.send_message(SourceToTargetMessage::StartFileBackup { sha256, file_name: info.name.to_string() })
        .await
        .change_context(ScheduledTaskError::Backup)?;

    let mut file = tokio::fs::File::open(tmp_db_path)
        .await
        .change_context(ScheduledTaskError::Backup)?;

    let buffer_size: usize = 1024 * 1024;
    let mut read_buffer: Vec<u8> = vec![0; buffer_size];
    let mut next_packet_number: Wrapping<u32> = Wrapping(0);

    loop {
        let size = file.read(&mut read_buffer)
            .await
            .change_context(ScheduledTaskError::Backup)?;
        let data = read_buffer[..size].to_vec();

        backup_client.send_message(SourceToTargetMessage::FileBackupData { package_number: next_packet_number, data })
            .await
            .change_context(ScheduledTaskError::Backup)?;

        next_packet_number += 1;

        if size == 0 {
            break;
        }
    }

    Ok(())
}

async fn calculate_hash(
    tmp_db_path: &str,
) -> Result<Sha256Bytes, ScheduledTaskError> {
    let mut hasher = Sha256::new();

    let mut file = tokio::fs::File::open(tmp_db_path)
        .await
        .change_context(ScheduledTaskError::Backup)?;

    let buffer_size: usize = 1024 * 1024;
    let mut read_buffer: Vec<u8> = vec![0; buffer_size];

    loop {
        let size = file.read(&mut read_buffer)
            .await
            .change_context(ScheduledTaskError::Backup)?;
        let data = &read_buffer[..size];
        hasher.update(data);

        if size == 0 {
            break;
        }
    }

    let result = hasher.finalize();
    Ok(Sha256Bytes(result.into()))
}

fn tmp_db_path_string(state: &S) -> Result<String, ScheduledTaskError> {
    state
        .config()
        .simple_backend()
        .data_dir()
        .join(DATABASE_BACKUP_TMP_FILE_NAME)
        .to_str()
        .map(|v| v.to_string())
        .ok_or(ScheduledTaskError::Backup.report())
}
