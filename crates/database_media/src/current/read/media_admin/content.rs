use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use model::{AccountId, ContentId};
use model_media::{
    AccountIdInternal, ContentModerationState, GetMediaContentFaceVerifiedNullList,
    MediaContentFaceVerifiedNullByAccount, MediaContentIterationPage, MediaContentIterationValue,
    MediaContentModerationQueuePage, MediaContentModerationQueueType, MediaContentModerationType,
    MediaContentPendingModeration, MediaContentRaw, MediaContentType,
};
use simple_backend_model::UnixTime;
use simple_backend_utils::Result;

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
            .order((media_content::upload_unix_time.asc(), account_id::id.asc()))
            .limit(LIMIT)
            .load::<MediaContentPendingModeration>(self.conn())
            .into_db_error(())?;

        Ok(MediaContentModerationQueuePage { values })
    }

    pub fn media_content_face_verified_null_list(
        &mut self,
    ) -> Result<GetMediaContentFaceVerifiedNullList, DieselDatabaseError> {
        use crate::schema::{
            account_id, current_account_media, data_consent_settings, media_content,
        };

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
            .inner_join(
                data_consent_settings::table
                    .on(current_account_media::account_id.eq(data_consent_settings::account_id)),
            )
            .filter(current_account_media::security_content_id.is_not_null())
            .filter(data_consent_settings::allow_face_verification.eq(true))
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

    pub fn media_content_iteration_page(
        &mut self,
        only_security_content: bool,
        page: i64,
        origin: UnixTime,
    ) -> Result<MediaContentIterationPage, DieselDatabaseError> {
        use crate::schema::{account_id, current_account_media, media_content};

        const PAGE_SIZE: i64 = 25;

        let is_security_content = diesel::dsl::exists(
            current_account_media::table
                .filter(
                    current_account_media::security_content_id
                        .assume_not_null()
                        .eq(media_content::id),
                )
                .filter(current_account_media::account_id.eq(media_content::account_id)),
        );

        let select_columns = (
            account_id::uuid,
            media_content::uuid,
            media_content::face_detected,
            media_content::face_detected_manual,
            media_content::secure_capture,
            media_content::upload_unix_time,
        );

        let from_clause = media_content::table
            .inner_join(account_id::table.on(media_content::account_id.eq(account_id::id)));

        type IterationRow = (AccountId, ContentId, bool, Option<bool>, bool, UnixTime);

        if only_security_content {
            let rows: Vec<IterationRow> = from_clause
                .filter(is_security_content)
                .filter(media_content::upload_unix_time.lt(origin.ut))
                .order((
                    media_content::upload_unix_time.desc(),
                    media_content::id.desc(),
                ))
                .select(select_columns)
                .limit(PAGE_SIZE)
                .offset(PAGE_SIZE.saturating_mul(page))
                .load(self.conn())
                .into_db_error(())?;

            return Ok(convert_iteration_rows(rows));
        }

        let rows: Vec<IterationRow> = from_clause
            .filter(media_content::upload_unix_time.lt(origin.ut))
            .order((
                media_content::upload_unix_time.desc(),
                media_content::id.desc(),
            ))
            .select(select_columns)
            .limit(PAGE_SIZE)
            .offset(PAGE_SIZE.saturating_mul(page))
            .load(self.conn())
            .into_db_error(())?;

        Ok(convert_iteration_rows(rows))
    }
}

/// Convert raw iteration row tuples into API values.
#[allow(clippy::type_complexity)]
fn convert_iteration_rows(
    rows: Vec<(AccountId, ContentId, bool, Option<bool>, bool, UnixTime)>,
) -> MediaContentIterationPage {
    let mut values = Vec::with_capacity(rows.len());

    for (
        account_id,
        content_id,
        face_detected,
        face_detected_manual,
        secure_capture,
        uploaded_time,
    ) in rows
    {
        values.push(MediaContentIterationValue {
            account_id,
            content_id,
            face_detected: face_detected_manual.unwrap_or(face_detected),
            secure_capture,
            uploaded_time,
        });
    }

    MediaContentIterationPage { values }
}
