use api_client::models::moderation;
use error_stack::{Result, ResultExt};

use sqlx::{Row, Sqlite, Transaction};

use crate::{
    api::{
        media::data::{
            ContentState, HandleModerationRequest, Moderation, ModerationId, ModerationRequestId,
            ModerationRequestQueueNumber, ModerationRequestState, CurrentAccountMediaInternal, MediaContentType,
        },
        model::{AccountIdInternal, ContentId, ModerationRequestContent},
    },
    server::data::{file::file::ImageSlot, sqlite::CurrentDataWriteHandle, write::WriteResult},
    utils::ConvertCommandError,
};

use super::super::super::sqlite::SqliteDatabaseError;

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
    ) -> WriteResult<Vec<Moderation>, SqliteDatabaseError> {
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
            .await
            .attach(moderator_id)?;

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
    /// Also updates content state.
    pub async fn update_moderation(
        &self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> WriteResult<(), SqliteDatabaseError, Moderation> {
        let account_row_id = moderation_request_owner.row_id();
        let request = sqlx::query!(
            r#"
            SELECT request_row_id
            FROM MediaModerationRequest
            WHERE account_row_id = ?
            "#,
            account_row_id,
        )
        .fetch_one(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let currently_selected_images =
            self.handle
                .read()
                .media()
                .get_current_account_media(moderation_request_owner)
                .await
                .convert(moderation_request_owner)
                .change_context(SqliteDatabaseError::Fetch)?;

        let moderation_id = ModerationId {
            request_id: ModerationRequestId {
                request_row_id: request.request_row_id,
            },
            account_id: moderator_id,
        };

        let content = self.handle.read().media().moderation(moderation_id).await?;

        let mut transaction = self
            .handle
            .pool()
            .begin()
            .await
            .into_error(SqliteDatabaseError::TransactionBegin)?;

        async fn actions(
            mut transaction: &mut Transaction<'_, Sqlite>,
            moderation_request_owner: AccountIdInternal,
            current_images_for_request_owner: CurrentAccountMediaInternal,
            moderation: ModerationId,
            state: ModerationRequestState,
            content: ModerationRequestContent,
        ) -> Result<(), SqliteDatabaseError> {
            let new_content_state = match state {
                ModerationRequestState::Accepted => ContentState::ModeratedAsAccepted,
                ModerationRequestState::Denied => ContentState::ModeratedAsDenied,
                ModerationRequestState::InProgress => ContentState::InModeration,
                ModerationRequestState::Waiting => ContentState::InSlot,
            };

            for c in content.content() {
                CurrentWriteMediaAdminCommands::update_content_state(
                    &mut transaction,
                    c,
                    new_content_state,
                    content.slot_1_is_security_image() && content.slot_1() == c
                )
                .await?;
            }

            if content.slot_1_is_security_image() &&
                state == ModerationRequestState::Accepted &&
                current_images_for_request_owner.security_content_id.is_none() {
                CurrentWriteMediaAdminCommands::update_current_security_image(transaction, moderation_request_owner, &content).await?;
                CurrentWriteMediaAdminCommands::update_current_primary_image_from_slot_2(transaction, moderation_request_owner, content).await?;
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

        let state = if result.accept {
            ModerationRequestState::Accepted
        } else {
            ModerationRequestState::Denied
        };

        match actions(&mut transaction, moderation_request_owner, currently_selected_images, moderation_id, state, content).await {
            Ok(()) => transaction
                .commit()
                .await
                .into_error(SqliteDatabaseError::TransactionCommit)
                .map_err(|e| e.into()),
            Err(e) => {
                match transaction
                    .rollback()
                    .await
                    .into_error(SqliteDatabaseError::TransactionRollback)
                {
                    Ok(()) => Err(e.into()),
                    Err(another_error) => Err(another_error.attach(e).into()),
                }
            }
        }
    }

    async fn update_current_security_image(
        transaction: &mut Transaction<'_, Sqlite>,
        moderation_request_owner: AccountIdInternal,
        content: &ModerationRequestContent,
    ) -> Result<(), SqliteDatabaseError> {
        let request_owner_id = moderation_request_owner.row_id();
        let security_img_content_id = content.slot_1().content_id;
        sqlx::query!(
            r#"
            UPDATE CurrentAccountMedia
            SET security_content_row_id = mc.content_row_id
            FROM (SELECT content_id, content_row_id FROM MediaContent) AS mc
            WHERE account_row_id = ? AND mc.content_id = ?
            "#,
            request_owner_id,
            security_img_content_id,
        )
        .execute(transaction)
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    async fn update_current_primary_image_from_slot_2(
        transaction: &mut Transaction<'_, Sqlite>,
        moderation_request_owner: AccountIdInternal,
        content: ModerationRequestContent,
    ) -> Result<(), SqliteDatabaseError> {
        let request_owner_id = moderation_request_owner.row_id();
        let primary_img_content_id = content.slot_2().ok_or(SqliteDatabaseError::ContentSlotEmpty)?.content_id;
        sqlx::query!(
            r#"
            UPDATE CurrentAccountMedia
            SET profile_content_row_id = mc.content_row_id
            FROM (SELECT content_id, content_row_id FROM MediaContent) AS mc
            WHERE account_row_id = ? AND mc.content_id = ?
            "#,
            request_owner_id,
            primary_img_content_id,
        )
        .execute(transaction)
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    async fn update_content_state(
        transaction: &mut Transaction<'_, Sqlite>,
        content_id: ContentId,
        new_state: ContentState,
        is_security: bool,
    ) -> Result<(), SqliteDatabaseError> {
        let state = new_state as i64;
        let content_type = if is_security {
            MediaContentType::Security as i64
        } else {
            MediaContentType::Normal as i64
        };

        sqlx::query!(
            r#"
            UPDATE MediaContent
            SET moderation_state = ?, content_type = ?
            WHERE content_id = ?
            "#,
            state,
            content_type,
            content_id,
        )
        .execute(transaction)
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}
