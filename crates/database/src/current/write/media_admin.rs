use error_stack::{Result};
use model::{
    AccountIdInternal, ContentId, ContentState,
    HandleModerationRequest, MediaContentType, Moderation, ModerationId, ModerationRequestId, ModerationQueueNumber,
    ModerationRequestState, ContentIdDb, PrimaryImage,
};

use utils::IntoReportExt;
use diesel::{prelude::*, delete, update};

use crate::{IntoDatabaseError, diesel::{DieselDatabaseError, DieselConnection}, current::{write::CurrentSyncWriteCommands}, TransactionError};

use super::media::CurrentSyncWriteMedia;

define_write_commands!(CurrentWriteMediaAdmin, CurrentSyncWriteMediaAdmin);

pub struct DeletedSomething;

impl<'a> CurrentSyncWriteMediaAdmin<'a> {
    fn delete_queue_number(
        &'a mut self,
        number: ModerationQueueNumber,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_moderation_queue_number::dsl::*;

        delete(
            media_moderation_queue_number
                .filter(queue_number.eq(number))
        )
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, number)?;

        Ok(())
    }

    pub fn moderation_get_list_and_create_new_if_necessary(
        &'a mut self,
        moderator_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DieselDatabaseError> {
        let mut moderations = self.cmds
            .read()
            .media_admin()
            .get_in_progress_moderations(moderator_id)?;

        const MAX_COUNT: usize = 5;
        if moderations.len() >= MAX_COUNT {
            return Ok(moderations);
        }

        let conn = self.conn();
        for _ in moderations.len()..MAX_COUNT {
            match CurrentSyncWriteCommands::new(conn)
                .media_admin()
                .create_moderation_from_next_request_in_queue(moderator_id)?
            {
                None => break,
                Some(moderation) => moderations.push(moderation),
            }
        }

        Ok(moderations)
    }

    fn create_moderation_from_next_request_in_queue(
        &'a mut self,
        moderator_id: AccountIdInternal,
    ) -> Result<Option<Moderation>, DieselDatabaseError> {
        // TODO: Really support multiple sub queues after account premium mode
        // is implemented.

        let id = self.cmds
            .read()
            .media_admin()
            .get_next_active_moderation_request(0, moderator_id)?;

        match id {
            None => Ok(None),
            Some(id) => {
                let moderation = self.create_moderation(id, moderator_id)?;
                Ok(Some(moderation))
            }
        }
    }

    fn create_moderation(
        &'a mut self,
        target_id: ModerationRequestId,
        moderator_id: AccountIdInternal,
    ) -> Result<Moderation, DieselDatabaseError> {
        // TODO: Currently is possible that two moderators moderate the same
        // request. Should that be prevented?

        let (content, queue_number, request_creator_id) = self.cmds
            .read()
            .media()
            .get_moderation_request_content(target_id)?;
        let content_string =
            serde_json::to_string(&content).into_error(DieselDatabaseError::SerdeSerialize)?;

        {
            use model::schema::media_moderation::dsl::*;
            diesel::insert_into(media_moderation)
                .values((
                    moderation_request_id.eq(target_id.request_row_id),
                    account_id.eq(moderator_id.as_db_id()),
                    state_number.eq(ModerationRequestState::InProgress as i64),
                    json_text.eq(content_string.clone()),
                ))
                .execute(self.cmds.conn)
                .into_db_error(DieselDatabaseError::Execute, (target_id, moderator_id))?;
        }

        self.delete_queue_number(queue_number)?;

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
    pub fn update_moderation(
        &'a mut self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<(), DieselDatabaseError> {
        //let conn = self.conn();
        let request =
            self.cmds.read()
                .media()
                .moderation_request(moderation_request_owner)?
                .ok_or(DieselDatabaseError::MissingModerationRequest)?;

        let currently_selected_images =
            //Self::read(conn)
            self.cmds.read()
                .media()
                .current_account_media(moderation_request_owner)?;

        let moderation_id = ModerationId {
            request_id: ModerationRequestId {
                request_row_id: request.moderation_request_id,
            },
            account_id: moderator_id,
        };

        let content =
            //Self::read(conn)
            self.cmds.read()
                .media_admin()
                .moderation(moderation_id)?;

        let state = if result.accept {
            ModerationRequestState::Accepted
        } else {
            ModerationRequestState::Denied
        };

        let new_content_state = match state {
            ModerationRequestState::Accepted => ContentState::ModeratedAsAccepted,
            ModerationRequestState::Denied => ContentState::ModeratedAsDenied,
            ModerationRequestState::InProgress => ContentState::InModeration,
            ModerationRequestState::Waiting => ContentState::InSlot,
        };

        self.conn()
            .transaction(|mut conn| {
                for c in content.content() {
                    CurrentSyncWriteMediaAdmin::update_content_state(
                        &mut conn,
                        c,
                        new_content_state,
                        content.slot_1_is_security_image() && content.slot_1() == c,
                    )?;
                }

                if content.slot_1_is_security_image()
                    && state == ModerationRequestState::Accepted
                    && currently_selected_images
                        .security_content_id
                        .is_none()
                {
                    CurrentSyncWriteMediaAdmin::update_current_security_image(
                        &mut conn,
                        moderation_request_owner,
                        content.slot_1(),
                    )?;

                    let primary_image = PrimaryImage {
                        content_id: content.slot_2(),
                        grid_crop_size: 0.0,
                        grid_crop_x: 0.0,
                        grid_crop_y: 0.0,
                    };

                    CurrentSyncWriteMedia::update_current_account_media_with_primary_image(
                        &mut conn,
                        moderation_request_owner,
                        primary_image,
                    )?;
                }

                let _state_number = state as i64;

                {
                    use model::schema::media_moderation::dsl::*;
                    update(media_moderation)
                        .filter(account_id.eq(moderation_id.account_id.as_db_id()))
                        .filter(moderation_request_id.eq(moderation_id.request_id.request_row_id))
                        .set(state_number.eq(state as i64))
                        .execute(conn)
                        .into_transaction_error(DieselDatabaseError::Execute, ())?;
                }

                Ok::<_, TransactionError<_>>(())
            })?;

            Ok(())
    }

    fn update_current_security_image(
        conn: &mut DieselConnection,
        moderation_request_owner: AccountIdInternal,
        image: ContentId,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;
        use model::schema::media_content;

        let content_id =
            media_content::table.filter(media_content::uuid.eq(image))
                .select(media_content::id)
                .first::<ContentIdDb>(conn)
                .into_db_error(DieselDatabaseError::Execute, (moderation_request_owner, image))?;

        update(current_account_media.find(moderation_request_owner.as_db_id()))
            .set((
                security_content_id.eq(content_id),
            ))
            .execute(conn)
            .into_db_error(DieselDatabaseError::Execute, (moderation_request_owner, image))?;

        Ok(())
    }

    // fn update_current_primary_image_from_slot_2(
    //     transaction_conn: &mut DieselConnection,
    //     moderation_request_owner: AccountIdInternal,
    //     content: ModerationRequestContent,
    // ) -> Result<(), DieselDatabaseError> {
    //     use model::schema::current_account_media::dsl::*;

    //     let request_owner_id = moderation_request_owner.row_id();
    //     let primary_img_content_id = content
    //         .slot_2()
    //         .ok_or(DieselDatabaseError::ContentSlotEmpty)?
    //         .content_id;



    //     update(media_content.filter(uuid.eq(content_id)))
    //         .set((
    //             moderation_state.eq(state),
    //             content_type.eq(content_type_value),
    //         ))
    //         .execute(transaction_conn)
    //         .into_db_error(DieselDatabaseError::Execute, (content_id, new_state))?;

    //     sqlx::query!(
    //         r#"
    //         UPDATE CurrentAccountMedia
    //         SET profile_content_row_id = mc.content_row_id
    //         FROM (SELECT content_id, content_row_id FROM MediaContent) AS mc
    //         WHERE account_row_id = ? AND mc.content_id = ?
    //         "#,
    //         request_owner_id,
    //         primary_img_content_id,
    //     )
    //     .execute(&mut **transaction)
    //     .await
    //     .into_error(SqliteDatabaseError::Execute)?;

    //     Ok(())
    // }

    fn update_content_state(
        transaction_conn: &mut DieselConnection,
        content_id: ContentId,
        new_state: ContentState,
        is_security: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        let state = new_state as i64;
        let content_type_value = if is_security {
            MediaContentType::Security as i64
        } else {
            MediaContentType::Normal as i64
        };

        update(media_content.filter(uuid.eq(content_id)))
            .set((
                moderation_state.eq(state),
                content_type.eq(content_type_value),
            ))
            .execute(transaction_conn)
            .into_db_error(DieselDatabaseError::Execute, (content_id, new_state))?;

        Ok(())
    }
}


// #[must_use]
// pub struct DatabaseTransaction<'a> {
//     transaction: Transaction<'a, Sqlite>,
// }

// impl<'a> DatabaseTransaction<'a> {
//     pub async fn store_content_id_to_slot(
//         pool: &'a sqlx::Pool<Sqlite>,
//         content_uploader: AccountIdInternal,
//         content_id: ContentId,
//         slot: ImageSlot,
//     ) -> error_stack::Result<DatabaseTransaction<'a>, SqliteDatabaseError> {
//         let content_uuid = content_id.as_uuid();
//         let account_row_id = content_uploader.row_id();
//         let state = ContentState::InSlot as i64;
//         let slot = slot as i64;

//         let mut transaction = pool
//             .begin()
//             .await
//             .into_error(SqliteDatabaseError::TransactionBegin)?;

//         sqlx::query!(
//             r#"
//             INSERT INTO MediaContent (content_id, account_row_id, moderation_state, slot_number)
//             VALUES (?, ?, ?, ?)
//             "#,
//             content_uuid,
//             account_row_id,
//             state,
//             slot,
//         )
//         .execute(&mut *transaction)
//         .await
//         .into_error(SqliteDatabaseError::Execute)?;

//         Ok(DatabaseTransaction { transaction })
//     }

//     pub async fn commit(self) -> error_stack::Result<(), SqliteDatabaseError> {
//         self.transaction
//             .commit()
//             .await
//             .into_error(SqliteDatabaseError::TransactionCommit)
//     }

//     pub async fn rollback(self) -> error_stack::Result<(), SqliteDatabaseError> {
//         self.transaction
//             .rollback()
//             .await
//             .into_error(SqliteDatabaseError::TransactionRollback)
//     }
// }
