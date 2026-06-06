use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::ContentId;
use model_media::{
    AccountIdInternal, ContentModerationState, GetMediaContentFaceVerifiedNullList,
    MediaContentFaceVerifiedNullByAccount, MediaContentModerationQueuePage,
    MediaContentModerationQueueType, MediaContentModerationType, MediaContentPendingModeration,
    MediaContentRaw, MediaContentType,
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadMediaAdminContent);

impl CurrentReadMediaAdminContent<'_> {
    pub fn media_content_moderation_queue_page(
        &mut self,
        content_type: MediaContentType,
        moderation_type: MediaContentModerationType,
        queue_type: MediaContentModerationQueueType,
    ) -> Result<MediaContentModerationQueuePage, DieselDatabaseError> {
        use crate::schema::{account_id, media_content};

        const LIMIT: i64 = 25;

        let initial_content_value = match moderation_type {
            MediaContentModerationType::Initial => true,
            MediaContentModerationType::Normal => false,
        };

        let states = match queue_type {
            MediaContentModerationQueueType::WaitingAdminBot => {
                [ContentModerationState::WaitingAdminBot].as_slice()
            }
            MediaContentModerationQueueType::WaitingAdmin => {
                [ContentModerationState::WaitingAdmin].as_slice()
            }
            MediaContentModerationQueueType::AcceptedByAdminBot => {
                [ContentModerationState::AcceptedByAdminBot].as_slice()
            }
            MediaContentModerationQueueType::RejectedByAdminBot => {
                [ContentModerationState::RejectedByAdminBot].as_slice()
            }
        };

        let values = media_content::table
            .inner_join(account_id::table.on(media_content::account_id.eq(account_id::id)))
            .filter(media_content::moderation_state.eq_any(states))
            .filter(media_content::content_type_number.eq(content_type))
            .filter(media_content::initial_content.eq(initial_content_value))
            .select((
                account_id::uuid,
                media_content::uuid,
                media_content::moderation_rejected_reason_category,
                media_content::moderation_rejected_reason_details,
            ))
            .order((
                media_content::creation_unix_time.asc(),
                account_id::id.asc(),
            ))
            .limit(LIMIT)
            .load::<MediaContentPendingModeration>(self.conn())
            .into_db_error(())?;

        Ok(MediaContentModerationQueuePage { values })
    }

    pub fn media_content_face_verified_null_list(
        &mut self,
    ) -> Result<GetMediaContentFaceVerifiedNullList, DieselDatabaseError> {
        use crate::schema::{account_id, current_account_media, media_content};

        const LIMIT: i64 = 25;

        let media_content_exists =
            diesel::alias!(crate::schema::media_content as media_content_exists);

        let has_face_verified_null_content = diesel::dsl::exists(
            media_content_exists
                .filter(
                    media_content_exists
                        .field(media_content::account_id)
                        .eq(current_account_media::account_id),
                )
                .filter(
                    media_content_exists
                        .field(media_content::face_verified)
                        .is_null(),
                )
                .filter(
                    media_content_exists
                        .field(media_content::face_detected)
                        .eq(true)
                        .or(media_content_exists
                            .field(media_content::face_detected_manual)
                            .eq(Some(true))),
                ),
        );

        let data = current_account_media::table
            .inner_join(account_id::table.on(current_account_media::account_id.eq(account_id::id)))
            .inner_join(
                media_content::table.on(current_account_media::security_content_id
                    .assume_not_null()
                    .eq(media_content::id)),
            )
            .filter(current_account_media::security_content_id.is_not_null())
            .filter(has_face_verified_null_content)
            .order((
                current_account_media::security_content_set_unix_time.asc(),
                account_id::id.asc(),
            ))
            .select((AccountIdInternal::as_select(), media_content::uuid))
            .limit(LIMIT)
            .load::<(AccountIdInternal, ContentId)>(self.conn())
            .into_db_error(())?;

        if data.is_empty() {
            return Ok(GetMediaContentFaceVerifiedNullList { values: vec![] });
        }

        let mut values = Vec::with_capacity(data.len());

        for (account_id_internal, security_content) in data {
            let account_values = media_content::table
                .filter(media_content::account_id.eq(account_id_internal.as_db_id()))
                .filter(media_content::face_verified.is_null())
                .filter(
                    media_content::face_detected
                        .eq(true)
                        .or(media_content::face_detected_manual.eq(Some(true))),
                )
                .select(MediaContentRaw::as_select())
                .load::<MediaContentRaw>(self.conn())
                .into_db_error(())?
                .into_iter()
                .map(|v| v.content_id())
                .collect();

            values.push(MediaContentFaceVerifiedNullByAccount {
                account_id: account_id_internal.as_id(),
                security_content,
                values: account_values,
            });
        }

        Ok(GetMediaContentFaceVerifiedNullList { values })
    }
}
