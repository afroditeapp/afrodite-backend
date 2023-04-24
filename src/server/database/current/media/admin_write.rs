



use error_stack::Result;

use sqlx::{Sqlite, Transaction, query::Query, Row, sqlite::SqliteRow};
use tracing::instrument::WithSubscriber;

use crate::{
    api::{
        media::data::{
            ContentState, Moderation, ModerationId, ModerationRequestId,
            ModerationRequestQueueNumber, ModerationRequestState,
        },
        model::{
            AccountIdInternal, ContentId,
            NewModerationRequest,
        },
    },
    server::database::{file::file::ImageSlot, sqlite::CurrentDataWriteHandle, current::media::write::CurrentWriteMediaCommands},
};

use super::super::super::sqlite::{SqliteDatabaseError};

use crate::utils::IntoReportExt;


#[must_use]
pub struct DatabaseTransaction<'a> {
    transaction: Transaction<'a, Sqlite>,
}

impl<'a> DatabaseTransaction<'a> {
    pub async fn store_content_id_to_slot(
        handle: &'a CurrentDataWriteHandle,
        content_uploader: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<DatabaseTransaction<'a>, SqliteDatabaseError> {
        let content_uuid = content_id.as_uuid();
        let account_row_id = content_uploader.row_id();
        let state = ContentState::InSlot as i64;
        let slot = slot as i64;

        let mut transaction = handle
            .pool()
            .begin()
            .await
            .into_error(SqliteDatabaseError::TransactionBegin)?;

        sqlx::query!(
            r#"
            INSERT INTO MediaContent (content_id, account_row_id, moderation_state, slot_number)
            VALUES (?, ?, ?, ?)
            "#,
            content_uuid,
            account_row_id,
            state,
            slot,
        )
        .execute(&mut transaction)
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(DatabaseTransaction { transaction })
    }

    pub async fn commit(self) -> Result<(), SqliteDatabaseError> {
        self.transaction
            .commit()
            .await
            .into_error(SqliteDatabaseError::TransactionCommit)
    }

    pub async fn rollback(self) -> Result<(), SqliteDatabaseError> {
        self.transaction
            .rollback()
            .await
            .into_error(SqliteDatabaseError::TransactionRollback)
    }
}

pub struct DeletedSomething;

pub struct CurrentWriteMediaAdminCommands<'a> {
    handle: &'a CurrentDataWriteHandle,
}

impl<'a> CurrentWriteMediaAdminCommands<'a> {
    pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
        Self { handle }
    }

    async fn delete_queue_number(
        &self,
        number: ModerationRequestQueueNumber,
    ) -> Result<(), SqliteDatabaseError> {
        sqlx::query!(
            r#"
            DELETE FROM MediaModerationQueueNumber
            WHERE queue_number = ?
            "#,
            number.number,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn moderation_get_list_and_create_new_if_necessary(
        &self,
        moderator_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, SqliteDatabaseError> {
        let mut moderations = self
            .handle
            .read()
            .media()
            .get_in_progress_moderations(moderator_id)
            .await?;
        const MAX_COUNT: usize = 5;
        if moderations.len() >= MAX_COUNT {
            return Ok(moderations);
        }

        for _ in moderations.len()..MAX_COUNT {
            match self
                .create_moderation_from_next_request_in_queue(moderator_id)
                .await?
            {
                None => break,
                Some(moderation) => moderations.push(moderation),
            }
        }

        Ok(moderations)
    }

    async fn create_moderation_from_next_request_in_queue(
        &self,
        moderator_id: AccountIdInternal,
    ) -> Result<Option<Moderation>, SqliteDatabaseError> {
        // TODO: Really support multiple sub queues after account premium mode
        // is implemented.

        let id = self
            .handle
            .read()
            .media()
            .get_next_active_moderation_request(0)
            .await?;

        match id {
            None => Ok(None),
            Some(id) => {
                let moderation = self.create_moderation(id, moderator_id).await?;
                Ok(Some(moderation))
            }
        }
    }

    async fn create_moderation(
        &self,
        target_id: ModerationRequestId,
        moderator_id: AccountIdInternal,
    ) -> Result<Moderation, SqliteDatabaseError> {
        // TODO: Currently is possible that two moderators moderate the same
        // request. Should that be prevented?

        let (content, queue_number, request_creator_id) = self
            .handle
            .read()
            .media()
            .get_moderation_request_content(target_id)
            .await?;
        let content_string =
            serde_json::to_string(&content).into_error(SqliteDatabaseError::SerdeSerialize)?;
        let account_row_id = moderator_id.row_id();
        let state = ModerationRequestState::InProgress as i64;

        sqlx::query!(
            r#"
            INSERT INTO MediaModeration (account_row_id, request_row_id, state_number, json_text)
            VALUES (?, ?, ?, ?)
            "#,
            account_row_id,
            target_id.request_row_id,
            state,
            content_string,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        self.delete_queue_number(queue_number).await?;

        let moderation = Moderation {
            request_creator_id,
            request_id: ModerationRequestId {
                request_row_id: target_id.request_row_id,
            },
            moderator_id: moderator_id.as_light(),
            content,
        };

        Ok(moderation)
    }

    /// Update moderation state of Moderation.
    ///
    /// Also updates
    pub async fn update_moderation(
        &self,
        moderation_id: ModerationId,
        state: ModerationRequestState,
    ) -> Result<(), SqliteDatabaseError> {
        let content = self.handle.read().media().moderation(moderation_id).await?;

        let mut transaction = self
            .handle
            .pool()
            .begin()
            .await
            .into_error(SqliteDatabaseError::TransactionBegin)?;

        async fn actions(
            transaction: &mut Transaction<'_, Sqlite>,
            moderation: ModerationId,
            state: ModerationRequestState,
            content: NewModerationRequest,
        ) -> Result<(), SqliteDatabaseError> {
            let new_content_state = match state {
                ModerationRequestState::Accepted => ContentState::ModeratedAsAccepted,
                ModerationRequestState::Denied => ContentState::ModeratedAsDenied,
                ModerationRequestState::InProgress => ContentState::InModeration,
                ModerationRequestState::Waiting => ContentState::InSlot,
            };

            for c in content.content() {
                CurrentWriteMediaAdminCommands::update_content_state(transaction, c, new_content_state)
                    .await?;
            }

            let state_number = state as i64;
            sqlx::query!(
                r#"
                UPDATE MediaModeration
                SET state_number = ?
                WHERE account_row_id = ? AND request_row_id = ?
                "#,
                state_number,
                moderation.account_id.account_row_id,
                moderation.request_id.request_row_id,
            )
            .execute(transaction)
            .await
            .into_error(SqliteDatabaseError::Execute)?;

            Ok(())
        }

        match actions(&mut transaction, moderation_id, state, content).await {
            Ok(()) => transaction
                .commit()
                .await
                .into_error(SqliteDatabaseError::TransactionCommit),
            Err(e) => {
                match transaction
                    .rollback()
                    .await
                    .into_error(SqliteDatabaseError::TransactionRollback)
                {
                    Ok(()) => Err(e),
                    Err(another_error) => Err(another_error.attach(e)),
                }
            }
        }
    }

    async fn update_content_state(
        transaction: &mut Transaction<'_, Sqlite>,
        content_id: ContentId,
        new_state: ContentState,
    ) -> Result<(), SqliteDatabaseError> {
        let state = new_state as i64;

        sqlx::query!(
            r#"
            UPDATE MediaContent
            SET moderation_state = ?
            WHERE content_id = ?
            "#,
            state,
            content_id.content_id,
        )
        .execute(transaction)
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

}
