use std::collections::HashSet;

use diesel::{prelude::*, backend::Backend};
use error_stack::{Result, ResultExt};
use model::{
    AccountId, AccountIdInternal, ContentId, ContentState,
    CurrentAccountMediaInternal, CurrentAccountMediaRaw, ContentSlot, MediaContentInternal,
    MediaContentRaw, MediaModerationRaw, ModerationQueueNumber, ModerationRequestContent,
    ModerationRequestId, ModerationRequestInternal, MediaModerationRequestRaw, ModerationRequestState, AccountIdDb, ContentIdDb, MediaContentType,
};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_read_commands!(CurrentReadMediaModerationRequest, CurrentSyncReadMediaModerationRequest);

impl<C: ConnectionProvider> CurrentSyncReadMediaModerationRequest<C> {
    pub fn moderation_request(
        &mut self,
        request_creator: AccountIdInternal,
    ) -> Result<Option<ModerationRequestInternal>, DieselDatabaseError> {
        let conn = self.conn();
        let request: MediaModerationRequestRaw = {
            use crate::schema::media_moderation_request::dsl::*;

            let request: Option<MediaModerationRequestRaw> = media_moderation_request
                .filter(account_id.eq(request_creator.as_db_id()))
                .select(MediaModerationRequestRaw::as_select())
                .first::<MediaModerationRequestRaw>(conn)
                .optional()
                .into_db_error(DieselDatabaseError::Execute, request_creator)?;

            match request {
                None => return Ok(None),
                Some(r) => r,
            }
        };

        use crate::schema::media_moderation::dsl::*;
        let moderations: Vec<MediaModerationRaw> = media_moderation
            .filter(moderation_request_id.eq(request.id))
            .select(MediaModerationRaw::as_select())
            .load(conn)
            .into_db_error(DieselDatabaseError::Execute, (request_creator, request.id))?;

        let state = match moderations.first() {
            None => ModerationRequestState::Waiting,
            Some(first) => {
                let accepted = moderations
                    .iter()
                    .find(|r| r.state_number == ModerationRequestState::Accepted as i64);
                let denied = moderations
                    .iter()
                    .find(|r| r.state_number == ModerationRequestState::Denied as i64);

                if let Some(accepted) = accepted {
                    ModerationRequestState::Accepted
                } else if let Some(denied) = denied {
                    ModerationRequestState::Denied
                } else {
                    ModerationRequestState::InProgress
                }
            }
        };

        let data: ModerationRequestContent = request.to_moderation_request_content();

        Ok(Some(ModerationRequestInternal::new(
            request.id,
            request_creator.as_id(),
            state,
            data,
        )))
    }

    pub fn get_media_content_from_slot(
        &mut self,
        slot_owner: AccountIdInternal,
        slot: ContentSlot,
    ) -> Result<Option<MediaContentInternal>, DieselDatabaseError> {
        let required_state = ContentState::InSlot as i64;
        let required_slot = slot as i64;

        let data: Option<MediaContentRaw> = {
            use crate::schema::media_content::dsl::*;

            media_content
                .filter(account_id.eq(slot_owner.as_db_id()))
                .filter(content_state.eq(required_state))
                .filter(slot_number.eq(required_slot))
                .select(MediaContentRaw::as_select())
                .first(self.conn())
                .optional()
                .into_db_error(DieselDatabaseError::Execute, (slot_owner, slot))?
        };

        Ok(data.map(|data| data.into()))
    }

    /// Validate moderation request content.
    ///
    /// Returns `Err(DieselDatabaseError::ModerationRequestContentInvalid)` if the
    /// content is invalid.
    pub fn content_validate_moderation_request_content(
        &mut self,
        content_owner: AccountIdInternal,
        request_content: &ModerationRequestContent,
    ) -> Result<(), DieselDatabaseError> {
        let requested_content_set: HashSet<ContentId> = request_content.content().collect();

        let required_state = ContentState::InSlot as i64;
        let data: Vec<MediaContentRaw> = {
            use crate::schema::media_content::dsl::*;

            media_content
                .filter(account_id.eq(content_owner.as_db_id()))
                .filter(content_state.eq(required_state))
                .select(MediaContentRaw::as_select())
                .load(self.conn())
                .into_db_error(DieselDatabaseError::Execute, content_owner)?
        };

        let database_content_set: HashSet<ContentId> = data.into_iter().map(|r| r.uuid).collect();

        if requested_content_set == database_content_set {
            Ok(())
        } else {
            Err(DieselDatabaseError::ModerationRequestContentInvalid)
                .with_info((content_owner, request_content))
        }
    }

    pub fn get_moderation_request_content(
        &mut self,
        owner_id: ModerationRequestId,
    ) -> Result<(MediaModerationRequestRaw, ModerationQueueNumber, AccountId), DieselDatabaseError>
    {
        let (request, account_id) = {
            use crate::schema::{
                account_id, media_moderation_request, media_moderation_request::dsl::*,
            };

            media_moderation_request::table
                .inner_join(account_id::table)
                .filter(id.eq(owner_id.request_row_id))
                .select((
                    MediaModerationRequestRaw::as_select(),
                    AccountIdInternal::as_select(),
                ))
                .first(self.conn())
                .into_db_error(DieselDatabaseError::Execute, owner_id)?
        };

        Ok((
            request.clone(),
            ModerationQueueNumber(request.queue_number),
            account_id.uuid,
        ))
    }
}

// --------------------------------------------------------------------
// Some code if future optimization is wanted:

// use crate::schema::current_account_media;
// use crate::schema::media_content;

// let (
//     security_content,
//     content_0,
//     content_1,
//     content_2,
//     content_3,
//     content_4,
//     content_5,
//     pending_security_content,
//     pending_content_0,
//     pending_content_1,
//     pending_content_2,
//     pending_content_3,
//     pending_content_4,
//     pending_content_5,
// ) = diesel::alias!(
//     media_content as security_content,
//     media_content as content_0,
//     media_content as content_1,
//     media_content as content_2,
//     media_content as content_3,
//     media_content as content_4,
//     media_content as content_5,
//     media_content as pending_security_content,
//     media_content as pending_content_0,
//     media_content as pending_content_1,
//     media_content as pending_content_2,
//     media_content as pending_content_3,
//     media_content as pending_content_4,
//     media_content as pending_content_5,
// );

// let (
//     raw,
//     s,
//     c0,
//     c1,
//     c2,
//     c3,
//     c4,
//     c5,
//     ps,
//     pc0,
//     pc1,
//     pc2,
//     pc3,
//     pc4,
//     pc5,
// ) =
//     current_account_media::table
//         .left_join(security_content.on(current_account_media::security_content_id.assume_not_null().eq(security_content.field(media_content::id))))
//         .left_join(content_0.on(current_account_media::profile_content_id_0.assume_not_null().eq(content_0.field(media_content::id))))
//         .left_join(content_1.on(current_account_media::profile_content_id_1.assume_not_null().eq(content_1.field(media_content::id))))
//         .left_join(content_2.on(current_account_media::profile_content_id_2.assume_not_null().eq(content_2.field(media_content::id))))
//         .left_join(content_3.on(current_account_media::profile_content_id_3.assume_not_null().eq(content_3.field(media_content::id))))
//         .left_join(content_4.on(current_account_media::profile_content_id_4.assume_not_null().eq(content_4.field(media_content::id))))
//         .left_join(content_5.on(current_account_media::profile_content_id_5.assume_not_null().eq(content_5.field(media_content::id))))
//         .left_join(pending_security_content.on(current_account_media::pending_security_content_id.assume_not_null().eq(pending_security_content.field(media_content::id))))
//         .left_join(pending_content_0.on(current_account_media::pending_profile_content_id_0.assume_not_null().eq(pending_content_0.field(media_content::id))))
//         .left_join(pending_content_1.on(current_account_media::pending_profile_content_id_1.assume_not_null().eq(pending_content_1.field(media_content::id))))
//         .left_join(pending_content_2.on(current_account_media::pending_profile_content_id_2.assume_not_null().eq(pending_content_2.field(media_content::id))))
//         .left_join(pending_content_3.on(current_account_media::pending_profile_content_id_3.assume_not_null().eq(pending_content_3.field(media_content::id))))
//         .left_join(pending_content_4.on(current_account_media::pending_profile_content_id_4.assume_not_null().eq(pending_content_4.field(media_content::id))))
//         .left_join(pending_content_5.on(current_account_media::pending_profile_content_id_5.assume_not_null().eq(pending_content_5.field(media_content::id))))
//         .filter(current_account_media::account_id.eq(media_owner_id.as_db_id()))
//         .select(
//             (
//                 CurrentAccountMediaRaw::as_select(),
//                 security_content.field(media_content::uuid).nullable(),
//                 content_0.field(media_content::uuid).nullable(),
//                 content_1.field(media_content::uuid).nullable(),
//                 content_2.field(media_content::uuid).nullable(),
//                 content_3.field(media_content::uuid).nullable(),
//                 content_4.field(media_content::uuid).nullable(),
//                 content_5.field(media_content::uuid).nullable(),
//                 pending_security_content.field(media_content::uuid).nullable(),
//                 pending_content_0.field(media_content::uuid).nullable(),
//                 pending_content_1.field(media_content::uuid).nullable(),
//                 pending_content_2.field(media_content::uuid).nullable(),
//                 pending_content_3.field(media_content::uuid).nullable(),
//                 pending_content_4.field(media_content::uuid).nullable(),
//                 pending_content_5.field(media_content::uuid).nullable(),
//             )
//         )
//         .first::<(
//             CurrentAccountMediaRaw,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//             Option<ContentId>,
//         )>(self.conn())
//         .into_db_error(DieselDatabaseError::Execute, media_owner_id)?;
