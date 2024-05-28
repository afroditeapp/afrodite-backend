use diesel::{delete, insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    AccountIdInternal, ContentId, ContentIdDb, ContentState, MediaContentRaw, MediaContentType,
    SetProfileContent, SetProfileContentInternal,
};
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_utils::ContextExt;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_write_commands!(CurrentWriteMediaContent, CurrentSyncWriteMediaContent);

impl<C: ConnectionProvider> CurrentSyncWriteMediaContent<C> {
    pub fn insert_current_account_media(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        insert_into(current_account_media)
            .values(account_id.eq(id.as_db_id()))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    /// Helper function for checking is ContentId valid to be set as current
    /// or pending content.
    ///
    /// Requirements:
    /// - The content must be in the `available` content list.
    /// - The content type must be JpegImage.
    /// - The provided check function must return true for the content.
    fn check_content_id(
        id: Option<ContentId>,
        available: &[MediaContentRaw],
        validate_state: impl Fn(&MediaContentRaw) -> bool,
    ) -> Result<Option<ContentIdDb>, DieselDatabaseError> {
        if let Some(content_id) = id {
            let found = available.iter().find(|content| {
                content.content_id() == content_id
                    && content.content_type() == MediaContentType::JpegImage
                    && validate_state(content)
            });

            if let Some(c) = found {
                Ok(Some(c.content_row_id()))
            } else {
                Err(DieselDatabaseError::NotAllowed.report())
            }
        } else {
            Ok(None)
        }
    }

    /// Update or delete current profile content if possible
    ///
    /// Requirements:
    ///  - The content must be moderated as accepted.
    ///  - The content must be of type JpegImage.
    ///  - The content must be in the account's media content.
    pub fn update_profile_content(
        &mut self,
        id: AccountIdInternal,
        new: SetProfileContent,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        let all_content = self
            .read()
            .media()
            .media_content()
            .get_account_media_content(id)?;
        let convert = |content_id: Option<ContentId>| {
            Self::check_content_id(content_id, &all_content, |c| {
                c.state() == ContentState::ModeratedAsAccepted
            })
        };

        update(current_account_media.find(id.as_db_id()))
            .set((
                profile_content_id_0.eq(convert(Some(new.content_id_0))?),
                profile_content_id_1.eq(convert(new.content_id_1)?),
                profile_content_id_2.eq(convert(new.content_id_2)?),
                profile_content_id_3.eq(convert(new.content_id_3)?),
                profile_content_id_4.eq(convert(new.content_id_4)?),
                profile_content_id_5.eq(convert(new.content_id_5)?),
                grid_crop_size.eq(new.grid_crop_size),
                grid_crop_x.eq(new.grid_crop_x),
                grid_crop_y.eq(new.grid_crop_y),
            ))
            .execute(self.conn())
            .into_db_error((id, new))?;

        Ok(())
    }

    /// Update or delete pending profile content if possible
    ///
    /// Requirements:
    ///  - The content must not be moderated as rejected.
    ///  - The content must be of type JpegImage.
    ///  - The content must be in the account's media content.
    pub fn update_or_delete_pending_profile_content(
        &mut self,
        id: AccountIdInternal,
        new: Option<SetProfileContent>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        let new: SetProfileContentInternal = if let Some(new) = new {
            // Update
            new.into()
        } else {
            // Delete
            SetProfileContentInternal::default()
        };

        let all_content = self
            .read()
            .media()
            .media_content()
            .get_account_media_content(id)?;
        let convert = |content_id: Option<ContentId>| {
            Self::check_content_id(content_id, &all_content, |c| {
                c.state() != ContentState::ModeratedAsRejected
            })
        };

        update(current_account_media.find(id.as_db_id()))
            .set((
                pending_profile_content_id_0.eq(convert(new.content_id_0)?),
                pending_profile_content_id_1.eq(convert(new.content_id_1)?),
                pending_profile_content_id_2.eq(convert(new.content_id_2)?),
                pending_profile_content_id_3.eq(convert(new.content_id_3)?),
                pending_profile_content_id_4.eq(convert(new.content_id_4)?),
                pending_profile_content_id_5.eq(convert(new.content_id_5)?),
                pending_grid_crop_size.eq(new.grid_crop_size),
                pending_grid_crop_x.eq(new.grid_crop_x),
                pending_grid_crop_y.eq(new.grid_crop_y),
            ))
            .execute(self.conn())
            .into_db_error((id, new))?;

        Ok(())
    }

    /// Update security content if possible
    ///
    /// Requirements:
    /// - The content must be moderated as accepted.
    /// - The content must be of type JpegImage.
    /// - The content must be in the account's media content.
    /// - The content must have secure capture flag enabled.
    pub fn update_security_content(
        &mut self,
        content_owner: AccountIdInternal,
        content_id: ContentId,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        let all_content = self
            .read()
            .media()
            .media_content()
            .get_account_media_content(content_owner)?;

        let content_db_id = Self::check_content_id(Some(content_id), &all_content, |c| {
            c.state() == ContentState::ModeratedAsAccepted && c.secure_capture
        })?;

        update(current_account_media.find(content_owner.as_db_id()))
            .set((security_content_id.eq(content_db_id),))
            .execute(self.conn())
            .into_db_error((content_owner, content_id))?;

        Ok(())
    }

    /// Update or delete pending security content if possible
    ///
    /// Requirements:
    /// - The content must not be moderated as rejected.
    /// - The content must be of type JpegImage.
    /// - The content must be in the account's media content.
    /// - The content must have secure capture flag enabled.
    pub fn update_or_delete_pending_security_content(
        &mut self,
        content_owner: AccountIdInternal,
        content_id: Option<ContentId>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        let all_content = self
            .read()
            .media()
            .media_content()
            .get_account_media_content(content_owner)?;

        let content_db_id = Self::check_content_id(content_id, &all_content, |c| {
            c.state() != ContentState::ModeratedAsRejected && c.secure_capture
        })?;

        update(current_account_media.find(content_owner.as_db_id()))
            .set((pending_security_content_id.eq(content_db_id),))
            .execute(self.conn())
            .into_db_error((content_owner, content_id))?;

        Ok(())
    }

    /// Delete content from account's media content if possible
    ///
    /// Requirements:
    /// - The content must be in the account's media content.
    /// - The content must not be set in account's current or pending
    ///   media content.
    /// - The content must not be in moderation.
    pub fn delete_content(
        &mut self,
        content_owner: AccountIdInternal,
        content_id: ContentId,
    ) -> Result<(), DieselDatabaseError> {
        let selected_content = self
            .read()
            .media()
            .media_content()
            .current_account_media(content_owner)?;
        let selected_content = selected_content
            .iter_all_content()
            .find(|c| c.content_id() == content_id);
        if selected_content.is_some() {
            return Err(DieselDatabaseError::ContentIsInUse.report());
        }

        let all_content = self
            .read()
            .media()
            .media_content()
            .get_account_media_content(content_owner)?;
        let found_content = all_content.iter().find(|c| c.content_id() == content_id);

        if let Some(c) = found_content {
            // TODO(prod): Content not in use time tracking
            match c.state() {
                ContentState::InSlot
                | ContentState::ModeratedAsRejected
                | ContentState::ModeratedAsAccepted => {
                    use model::schema::media_content::dsl::*;
                    delete(media_content.filter(id.eq(c.content_row_id())))
                        .execute(self.conn())
                        .into_db_error((content_owner, content_id))?;
                    Ok(())
                }
                ContentState::InModeration => Err(DieselDatabaseError::ContentIsInUse.report()),
            }
        } else {
            Err(DieselDatabaseError::NotAllowed.report())
        }
    }

    pub fn move_pending_content_to_current_content(
        &mut self,
        content_owner: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;
        let c = self
            .read()
            .media()
            .media_content()
            .current_account_media_raw(content_owner)?;

        // TODO(prod): Handle case where user creates initial moderation request,
        //             uploads new image, and then updates pending content
        //             with the new image. The current code seems to allow
        //             that case which means that not moderated image can be
        //             set as current content.
        //             Fix: Check current content states here and do not allow
        //             non moderated content to be set as current content.
        //             Make also test for this case.

        // TODO(prod): Check also that required security and primary content really is
        // set in pending content.

        update(current_account_media.find(content_owner.as_db_id()))
            .set((
                security_content_id.eq(c.pending_security_content_id),
                profile_content_id_0.eq(c.pending_profile_content_id_0),
                profile_content_id_1.eq(c.pending_profile_content_id_1),
                profile_content_id_2.eq(c.pending_profile_content_id_2),
                profile_content_id_3.eq(c.pending_profile_content_id_3),
                profile_content_id_4.eq(c.pending_profile_content_id_4),
                profile_content_id_5.eq(c.pending_profile_content_id_5),
                grid_crop_size.eq(c.pending_grid_crop_size),
                grid_crop_x.eq(c.pending_grid_crop_x),
                grid_crop_y.eq(c.pending_grid_crop_y),
                pending_security_content_id.eq(None::<ContentIdDb>),
                pending_profile_content_id_0.eq(None::<ContentIdDb>),
                pending_profile_content_id_1.eq(None::<ContentIdDb>),
                pending_profile_content_id_2.eq(None::<ContentIdDb>),
                pending_profile_content_id_3.eq(None::<ContentIdDb>),
                pending_profile_content_id_4.eq(None::<ContentIdDb>),
                pending_profile_content_id_5.eq(None::<ContentIdDb>),
                pending_grid_crop_size.eq(None::<f64>),
                pending_grid_crop_x.eq(None::<f64>),
                pending_grid_crop_y.eq(None::<f64>),
            ))
            .execute(self.conn())
            .into_db_error(content_owner)?;

        Ok(())
    }
}
