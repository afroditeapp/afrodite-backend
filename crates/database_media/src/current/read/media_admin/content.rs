use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use model::{AccountId, ContentId, ContentIdDb, ProfileAge};
use model_media::{
    AccountIdInternal, ContentModerationState, GetMediaContentFaceVerifiedNullList,
    MediaContentFaceVerifiedNullByAccount, MediaContentIdDbValue, MediaContentIterationPage,
    MediaContentIterationValue, MediaContentIteratorPageData, MediaContentIteratorSettingsInternal,
    MediaContentModerationQueuePage, MediaContentModerationQueueType,
    MediaContentModerationRejectedReasonCategory, MediaContentModerationRejectedReasonDetails,
    MediaContentModerationType, MediaContentPendingModeration, MediaContentRaw, MediaContentType,
};
use simple_backend_model::{NonEmptyString, UnixTime};
use simple_backend_utils::Result;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadMediaAdminContent);

impl CurrentReadMediaAdminContent<'_> {
    /// Get all media content of an account with the moderator account ID
    /// resolved to the public account ID.
    pub fn all_account_media_content_admin(
        &mut self,
        media_owner_id: AccountIdInternal,
    ) -> Result<Vec<(MediaContentRaw, Option<AccountId>)>, DieselDatabaseError> {
        use crate::schema::{account_id, media_content};

        let moderator_aid = diesel::alias!(crate::schema::account_id as moderator_aid);

        media_content::table
            .left_join(
                moderator_aid.on(media_content::moderation_moderator_account_id
                    .eq(moderator_aid.field(account_id::id).nullable())),
            )
            .filter(media_content::account_id.eq(media_owner_id.as_db_id()))
            .select((
                MediaContentRaw::as_select(),
                moderator_aid.field(account_id::uuid).nullable(),
            ))
            .load(self.conn())
            .into_db_error(media_owner_id)
    }

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

    pub fn get_latest_media_content_id_db(
        &mut self,
    ) -> Result<MediaContentIdDbValue, DieselDatabaseError> {
        use crate::schema::media_content;

        let content_id_db: ContentIdDb = media_content::table
            .order(media_content::id.desc())
            .select(media_content::id)
            .first(self.conn())
            .into_db_error(())?;

        Ok(MediaContentIdDbValue { content_id_db })
    }

    /// Also returns if there are more pages after the requested page.
    pub fn media_content_iterator_page(
        &mut self,
        settings: MediaContentIteratorSettingsInternal,
    ) -> Result<(Vec<MediaContentIteratorPageData>, bool), DieselDatabaseError> {
        use crate::schema::{account_id, media_content, profile};

        const PAGE_SIZE: i64 = 25;

        let moderator_aid = diesel::alias!(crate::schema::account_id as moderator_aid);

        let filters = settings.filters;

        let mut query = media_content::table
            .inner_join(account_id::table.on(media_content::account_id.eq(account_id::id)))
            .inner_join(profile::table.on(media_content::account_id.eq(profile::account_id)))
            .left_join(
                moderator_aid.on(media_content::moderation_moderator_account_id
                    .eq(moderator_aid.field(account_id::id).nullable())),
            )
            .select((
                account_id::uuid,
                media_content::uuid,
                profile::profile_name,
                profile::age,
                media_content::upload_unix_time,
                media_content::secure_capture,
                media_content::face_detected,
                media_content::face_detected_manual,
                media_content::initial_content,
                media_content::moderation_state,
                media_content::moderation_rejected_reason_category,
                media_content::moderation_rejected_reason_details,
                moderator_aid.field(account_id::uuid).nullable(),
            ))
            .filter(media_content::id.le(settings.start_position))
            .order(media_content::id.desc())
            .into_boxed();

        // Empty list is treated as no filter.
        if let Some(states) = filters
            .moderation_states
            .filter(|states| !states.is_empty())
        {
            query = query.filter(media_content::moderation_state.eq_any(states));
        }

        if let Some(value) = filters.initial_content {
            query = query.filter(media_content::initial_content.eq(value));
        }

        if let Some(value) = filters.secure_capture {
            query = query.filter(media_content::secure_capture.eq(value));
        }

        if let Some(in_use) = filters.in_use {
            query = if in_use {
                query
                    .filter(media_content::usage_start_unix_time.is_not_null())
                    .filter(media_content::usage_end_unix_time.is_null())
            } else {
                query.filter(
                    media_content::usage_start_unix_time
                        .is_null()
                        .or(media_content::usage_end_unix_time.is_not_null()),
                )
            };
        }

        if let Some(time) = filters.uploaded_after_unix_time {
            query = query.filter(media_content::upload_unix_time.gt(time));
        }

        if let Some(time) = filters.uploaded_before_unix_time {
            query = query.filter(media_content::upload_unix_time.lt(time));
        }

        if let Some(value) = filters.face_detected {
            // Manual value overrides the automatic value.
            query = query.filter(
                media_content::face_detected_manual.eq(Some(value)).or(
                    media_content::face_detected_manual
                        .is_null()
                        .and(media_content::face_detected.eq(value)),
                ),
            );
        }

        if let Some(value) = filters.face_detected_manual_set {
            query = if value {
                query.filter(media_content::face_detected_manual.is_not_null())
            } else {
                query.filter(media_content::face_detected_manual.is_null())
            };
        }

        if let Some(id) = filters.moderator_account_id {
            query = query.filter(media_content::moderation_moderator_account_id.eq(id));
        }

        if let Some(id) = filters.moderator_account_id_not {
            query = query.filter(
                media_content::moderation_moderator_account_id
                    .ne(id)
                    .or(media_content::moderation_moderator_account_id.is_null()),
            );
        }

        // Fetch one extra row to detect if there are more pages after
        // the requested page.
        let limit = PAGE_SIZE.saturating_add(1);

        #[allow(clippy::type_complexity)]
        let data: Vec<(
            AccountId,
            ContentId,
            Option<NonEmptyString>,
            ProfileAge,
            UnixTime,
            bool,
            bool,
            Option<bool>,
            bool,
            ContentModerationState,
            Option<MediaContentModerationRejectedReasonCategory>,
            Option<MediaContentModerationRejectedReasonDetails>,
            Option<AccountId>,
        )> = query
            .limit(limit)
            .offset(PAGE_SIZE.saturating_mul(settings.page))
            .load(self.conn())
            .into_db_error(())?;

        let has_more_pages = data.len() > PAGE_SIZE.try_into().unwrap_or(usize::MAX);

        Ok((
            data.into_iter()
                .take(PAGE_SIZE.try_into().unwrap_or(usize::MAX))
                .map(
                    |(
                        account_id,
                        content_id,
                        name,
                        age,
                        upload_time,
                        secure_capture,
                        face_detected,
                        face_detected_manual,
                        initial_content,
                        moderation_state,
                        rejected_reason_category,
                        rejected_reason_details,
                        moderator_account_id,
                    )| MediaContentIteratorPageData {
                        account_id,
                        content_id,
                        name,
                        age,
                        upload_time,
                        secure_capture,
                        face_detected: face_detected_manual.unwrap_or(face_detected),
                        initial_content,
                        moderation_state,
                        rejected_reason_category,
                        rejected_reason_details,
                        moderator_account_id,
                    },
                )
                .collect(),
            has_more_pages,
        ))
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
