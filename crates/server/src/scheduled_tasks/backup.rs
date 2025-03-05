

use std::sync::atomic::{AtomicU32, Ordering};

use manager_model::{AccountAndContent, SourceToTargetMessage, TargetToSourceMessage};
use model::{AccountId, ContentId};
use server_api::{
    app::ReadData,
    result::WrappedContextExt,
};
use server_common::result::{Result, WrappedResultExt};
use server_data::read::GetReadCommandsCommon;
use server_data_media::read::GetReadMediaCommands;
use server_state::S;
use simple_backend::ServerQuitWatcher;
use simple_backend::app::GetManagerApi;
use tokio::sync::broadcast::error::TryRecvError;

use super::ScheduledTaskError;

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
                    backup_client.send_message(SourceToTargetMessage::ContentQueryAnswer { data })
                        .await
                        .change_context(ScheduledTaskError::Backup)?;
                }
            }
        }
    }

    backup_client.send_message(SourceToTargetMessage::ContentList { data: vec![] })
        .await
        .change_context(ScheduledTaskError::Backup)?;

    // TODO(prod): SQLite database backups

    Ok(())
}
