use std::sync::atomic::{AtomicU32, Ordering};

use manager_api::backup::BackupSourceClient;
use manager_model::{AccountAndContent, Sha256Bytes, SourceToTargetMessage, TargetToSourceMessage};
use model::{AccountId, ContentId, ContentQualityVariant};
use server_api::{
    DataError,
    app::{GetConfig, ReadData},
    result::WrappedContextExt,
};
use server_common::{
    backup_encryption::{encrypt_backup_data, encrypt_backup_data_stream},
    result::{Result, WrappedResultExt},
};
use server_data::read::GetReadCommandsCommon;
use server_data_media::read::GetReadMediaCommands;
use server_state::S;
use sha2::{Digest, Sha256};
use simple_backend::{ServerQuitWatcher, app::GetManagerApi};
use simple_backend_config::Database;
use simple_backend_utils::{consts::MIB_IN_BYTES, file::overwrite_and_remove_if_exists};
use tokio::{io::AsyncReadExt, sync::broadcast::error::TryRecvError};

use super::ScheduledTaskError;

const DATABASE_BACKUP_TMP_FILE_NAME: &str = "database_backup.tmp";

static BACKUP_SESSION: AtomicU32 = AtomicU32::new(0);

pub async fn backup_data(
    state: &S,
    quit_notification: &mut ServerQuitWatcher,
) -> Result<(), ScheduledTaskError> {
    let Some((mut backup_client, encryption_key)) = state
        .manager_api_client()
        .new_backup_connection(BACKUP_SESSION.fetch_add(1, Ordering::Relaxed))
        .await
        .change_context(ScheduledTaskError::Backup)?
    else {
        // Backup link password is not configured
        return Ok(());
    };

    backup_client
        .send_message(SourceToTargetMessage::StartBackupSession)
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

        backup_client
            .send_message(SourceToTargetMessage::ContentList { data })
            .await
            .change_context(ScheduledTaskError::Backup)?;

        loop {
            let m = backup_client
                .receive_message()
                .await
                .change_context(ScheduledTaskError::Backup)?;

            match m {
                TargetToSourceMessage::ContentListSyncDone => break,
                TargetToSourceMessage::ContentQuery {
                    account_id,
                    content_id,
                    high,
                    medium,
                    low,
                } => {
                    let read_variant_data = async |variant| -> Result<
                        (ContentQualityVariant, Sha256Bytes, Vec<u8>),
                        ScheduledTaskError,
                    > {
                        let content_data = state
                            .read()
                            .media()
                            .content_data_variant(
                                AccountId { aid: account_id },
                                ContentId { cid: content_id },
                                variant,
                            )
                            .change_context(ScheduledTaskError::DatabaseError)?;
                        let plaintext = content_data
                            .read_all()
                            .await
                            .change_context(ScheduledTaskError::FileReadingError)?;
                        let encrypted = encrypt_backup_data(&encryption_key.0, &plaintext);
                        let mut hasher = Sha256::new();
                        hasher.update(&encrypted);
                        let result = hasher.finalize();
                        Ok((variant, Sha256Bytes(result.into()), encrypted))
                    };

                    let mut variants = vec![];
                    if high {
                        variants.push(read_variant_data(ContentQualityVariant::High).await?);
                    }
                    if medium {
                        variants.push(read_variant_data(ContentQualityVariant::Medium).await?);
                    }
                    if low {
                        variants.push(read_variant_data(ContentQualityVariant::Low).await?);
                    }

                    backup_client
                        .send_message(SourceToTargetMessage::ContentQueryAnswer(variants))
                        .await
                        .change_context(ScheduledTaskError::Backup)?;
                }
            }
        }
    }

    // Empty file name ends content backup waiting
    backup_client
        .send_message(SourceToTargetMessage::ContentList { data: vec![] })
        .await
        .change_context(ScheduledTaskError::Backup)?;

    let tmp_db = tmp_db_path_string(state)?;

    let databases = state.config().simple_backend().database_info();

    handle_db(
        &mut backup_client,
        &tmp_db,
        &databases.current,
        &encryption_key.0,
        state
            .read()
            .common()
            .backup_current_database(tmp_db.clone()),
    )
    .await?;
    handle_db(
        &mut backup_client,
        &tmp_db,
        &databases.history,
        &encryption_key.0,
        state
            .read()
            .common_history()
            .backup_history_database(tmp_db.clone()),
    )
    .await?;

    // Empty file name ends file backup waiting
    backup_client
        .send_message(SourceToTargetMessage::StartFileBackup {
            file_name: String::new(),
        })
        .await
        .change_context(ScheduledTaskError::Backup)?;

    Ok(())
}

async fn handle_db(
    backup_client: &mut BackupSourceClient,
    tmp_db: &str,
    db_name: &Database,
    key: &[u8; 16],
    create_backup_file: impl Future<Output = Result<(), DataError>>,
) -> Result<(), ScheduledTaskError> {
    overwrite_and_remove_if_exists(tmp_db)
        .await
        .change_context(ScheduledTaskError::Backup)?;

    create_backup_file
        .await
        .change_context(ScheduledTaskError::DatabaseError)?;

    send_backup_db(db_name, tmp_db, key, backup_client).await?;

    overwrite_and_remove_if_exists(tmp_db)
        .await
        .change_context(ScheduledTaskError::Backup)?;

    Ok(())
}

async fn send_backup_db(
    info: &Database,
    tmp_db_path: &str,
    key: &[u8; 16],
    backup_client: &mut BackupSourceClient,
) -> Result<(), ScheduledTaskError> {
    backup_client
        .send_message(SourceToTargetMessage::StartFileBackup {
            file_name: info.sqlite_name().to_string(),
        })
        .await
        .change_context(ScheduledTaskError::Backup)?;

    let mut stream = EncryptedBackupFileStream::open(tmp_db_path, key).await?;

    loop {
        let Some(data) = stream.next_packet().await? else {
            break;
        };

        backup_client
            .send_message(SourceToTargetMessage::FileBackupData { data })
            .await
            .change_context(ScheduledTaskError::Backup)?;
    }

    let sha256 = stream.finalize_hash();

    backup_client
        .send_message(SourceToTargetMessage::EndFileBackup { sha256 })
        .await
        .change_context(ScheduledTaskError::Backup)?;

    Ok(())
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

struct EncryptedBackupFileStream<'a> {
    file: tokio::fs::File,
    key: &'a [u8; 16],
    hasher: Sha256,
    buffer: Vec<u8>,
    is_first: bool,
}

impl<'a> EncryptedBackupFileStream<'a> {
    async fn open(path: &str, key: &'a [u8; 16]) -> Result<Self, ScheduledTaskError> {
        let file = tokio::fs::File::open(path)
            .await
            .change_context(ScheduledTaskError::Backup)?;
        Ok(Self {
            file,
            key,
            hasher: Sha256::new(),
            buffer: vec![0; MIB_IN_BYTES],
            is_first: true,
        })
    }

    /// Read next chunk from file, encrypt it, update SHA-256.
    /// Returns `None` on EOF.
    async fn next_packet(&mut self) -> Result<Option<Vec<u8>>, ScheduledTaskError> {
        let size = self
            .file
            .read(&mut self.buffer)
            .await
            .change_context(ScheduledTaskError::Backup)?;
        if size == 0 {
            return Ok(None);
        }
        let plaintext = &self.buffer[..size];
        let is_first = self.is_first;
        self.is_first = false;
        let encrypted = encrypt_backup_data_stream(self.key, plaintext, is_first);
        self.hasher.update(&encrypted);
        Ok(Some(encrypted))
    }

    fn finalize_hash(self) -> Sha256Bytes {
        Sha256Bytes(self.hasher.finalize().into())
    }
}
