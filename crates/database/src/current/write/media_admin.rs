use diesel::{delete, prelude::*, update};
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, ContentId, ContentIdDb, ContentState, HandleModerationRequest,
    Moderation, ModerationId, ModerationQueueNumber, ModerationRequestId,
    ModerationRequestState, ProfileContent, NextQueueNumberType, schema::media_moderation_request::content_id_1,
};
use simple_backend_database::diesel_db::{DieselConnection, DieselDatabaseError};

use super::{media::CurrentSyncWriteMedia, ConnectionProvider};
use crate::{IntoDatabaseError, TransactionError, current::write::CurrentSyncWriteCommands};

define_write_commands!(CurrentWriteMediaAdmin, CurrentSyncWriteMediaAdmin);

pub struct DeletedSomething;

impl<C: ConnectionProvider> CurrentSyncWriteMediaAdmin<C> {

    pub fn moderation_get_list_and_create_new_if_necessary(
        &mut self,
        moderator_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DieselDatabaseError> {
        let mut moderations = self
            .read()
            .media_admin()
            .get_in_progress_moderations(moderator_id)?;

        const MAX_COUNT: usize = 5;
        if moderations.len() >= MAX_COUNT {
            return Ok(moderations);
        }

        for _ in moderations.len()..MAX_COUNT {
            match self.cmds()
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
        &mut self,
        moderator_id: AccountIdInternal,
    ) -> Result<Option<Moderation>, DieselDatabaseError> {
        let id = self
            .read()
            .media_admin()
            .get_next_active_moderation_request(true, moderator_id)?;

        match id {
            None => Ok(None),
            Some(id) => {
                let moderation = self.create_moderation(id, moderator_id)?;
                Ok(Some(moderation))
            }
        }
    }

    fn create_moderation(
        &mut self,
        target_id: ModerationRequestId,
        moderator_id: AccountIdInternal,
    ) -> Result<Moderation, DieselDatabaseError> {
        // TODO: Currently is possible that two moderators moderate the same
        // request. Should that be prevented?

        let (request_raw, queue_number, request_creator_id) = self
            .read()
            .media()
            .get_moderation_request_content(target_id)?;
        let content = request_raw.to_moderation_request_content();
        let content_string =
            serde_json::to_string(&content).change_context(DieselDatabaseError::SerdeSerialize)?;

        {
            use model::schema::media_moderation::dsl::*;
            diesel::insert_into(media_moderation)
                .values((
                    moderation_request_id.eq(target_id.request_row_id),
                    account_id.eq(moderator_id.as_db_id()),
                    state_number.eq(ModerationRequestState::InProgress as i64),
                ))
                .execute(self.cmds.conn())
                .into_db_error(DieselDatabaseError::Execute, (target_id, moderator_id))?;
        }

        // TODO
        let queue_type = if true { //if request_raw.initial_moderation_security_image.is_some() {
            NextQueueNumberType::InitialMediaModeration
        } else {
            NextQueueNumberType::MediaModeration
        };
        self.cmds().common().delete_queue_entry(queue_number.0, queue_type)?;

        let moderation = Moderation {
            request_creator_id,
            request_id: ModerationRequestId {
                request_row_id: target_id.request_row_id,
            },
            moderator_id: moderator_id.as_id(),
            content,
        };

        Ok(moderation)
    }

    /// Update moderation state of Moderation.
    ///
    /// Also updates content state.
    pub fn update_moderation(
        &mut self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<(), DieselDatabaseError> {
        let request = self
            .read()
            .media()
            .moderation_request(moderation_request_owner)?
            .ok_or(DieselDatabaseError::MissingModerationRequest)?;

        let currently_selected_images = self
            .read()
            .media()
            .current_account_media(moderation_request_owner)?;

        let moderation_id = ModerationId {
            request_id: ModerationRequestId {
                request_row_id: request.moderation_request_id,
            },
            account_id: moderator_id,
        };

        let content = self.read().media_admin().moderation(moderation_id)?;

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

        for c in content.content() {
            let content_info = self.read().media().get_media_content_raw(c)?;
            // TODO
            //let is_security = if let Some(content) = content.initial_moderation_security_image {
            let is_security = if true {
                //content == c
                true
            } else {
                false
            };
            self.update_content_state(
                c,
                new_content_state,
                // is_security,
            )?;
        }

        // TODO
        // if let Some(security_image) = content.initial_moderation_security_image {
        //     if state == ModerationRequestState::Accepted
        //     && currently_selected_images.security_content_id.is_none() {
        //         self.update_current_security_image(
        //             moderation_request_owner,
        //             security_image,
        //         )?;

        //         let primary_image = PrimaryImage {
        //             //content_id: Some(content.content1),
        //             content_id: Some(content.content0),
        //             grid_crop_size: 0.0,
        //             grid_crop_x: 0.0,
        //             grid_crop_y: 0.0,
        //         };

        //         self.cmds().media().update_current_account_media_with_primary_image(
        //             moderation_request_owner,
        //             primary_image,
        //         )?;
        //     }

        // }

        let _state_number = state as i64;

        {
            use model::schema::media_moderation::dsl::*;
            update(media_moderation)
                .filter(account_id.eq(moderation_id.account_id.as_db_id()))
                .filter(moderation_request_id.eq(moderation_id.request_id.request_row_id))
                .set(state_number.eq(state as i64))
                .execute(self.conn())
                .into_transaction_error(DieselDatabaseError::Execute, ())?;
        }

        Ok(())
    }

    fn update_current_security_image(
        &mut self,
        moderation_request_owner: AccountIdInternal,
        image: ContentId,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::{current_account_media::dsl::*, media_content};

        let content_id = media_content::table
            .filter(media_content::uuid.eq(image))
            .select(media_content::id)
            .first::<ContentIdDb>(self.conn())
            .into_db_error(
                DieselDatabaseError::Execute,
                (moderation_request_owner, image),
            )?;

        update(current_account_media.find(moderation_request_owner.as_db_id()))
            .set((security_content_id.eq(content_id),))
            .execute(self.conn())
            .into_db_error(
                DieselDatabaseError::Execute,
                (moderation_request_owner, image),
            )?;

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
    //     .change_context(SqliteDatabaseError::Execute)?;

    //     Ok(())
    // }

    fn update_content_state(
        &mut self,
        content_id: ContentId,
        new_state: ContentState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        update(media_content.filter(uuid.eq(content_id)))
            .set((
                content_state.eq(new_state),
            ))
            .execute(self.conn())
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
//         slot: ContentSlot,
//     ) -> error_stack::Result<DatabaseTransaction<'a>, SqliteDatabaseError> {
//         let content_uuid = content_id.as_uuid();
//         let account_row_id = content_uploader.row_id();
//         let state = ContentState::InSlot as i64;
//         let slot = slot as i64;

//         let mut transaction = pool
//             .begin()
//             .await
//             .change_context(SqliteDatabaseError::TransactionBegin)?;

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
//         .change_context(SqliteDatabaseError::Execute)?;

//         Ok(DatabaseTransaction { transaction })
//     }

//     pub async fn commit(self) -> error_stack::Result<(), SqliteDatabaseError> {
//         self.transaction
//             .commit()
//             .await
//             .change_context(SqliteDatabaseError::TransactionCommit)
//     }

//     pub async fn rollback(self) -> error_stack::Result<(), SqliteDatabaseError> {
//         self.transaction
//             .rollback()
//             .await
//             .change_context(SqliteDatabaseError::TransactionRollback)
//     }
// }
