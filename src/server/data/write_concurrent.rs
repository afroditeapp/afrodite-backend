//! Write commands that can be run concurrently also with synchronous
//! write commands.


use std::{fmt::Debug, marker::PhantomData, net::SocketAddr};

use axum::extract::BodyStream;
use error_stack::{Report, Result, ResultExt};

use crate::server::data::database::current::CurrentDataWriteCommands;
use crate::{
    api::{
        media::data::{HandleModerationRequest, Moderation, PrimaryImage},
        model::{
            Account, AccountIdInternal, AccountIdLight, AccountSetup, AuthPair, ContentId,
            Location, ModerationRequestContent, ProfileLink, SignInWithInfo,
        },
    },
    config::Config,
    media_backup::MediaBackupHandle,
    server::data::DatabaseError,
    utils::{ConvertCommandError, ErrorConversion},
};

use super::{
    cache::{CacheError, CachedProfile, DatabaseCache, WriteCacheJson},
    database::history::write::HistoryWriteCommands,
    database::sqlite::{
        CurrentDataWriteHandle, HistoryUpdateJson, HistoryWriteHandle, SqliteDatabaseError,
        SqliteUpdateJson,
    },
    file::{file::ImageSlot, utils::FileDir},
    index::{LocationIndexIteratorGetter, LocationIndexWriterGetter},
};




/// Commands that can run concurrently with other write commands, but which have
/// limitation that one account can execute only one command at a time.
/// It possible to run this and normal write command concurrently for
/// one account.
pub struct WriteCommandsConcurrent<'a> {
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location: LocationIndexIteratorGetter<'a>,
}

impl<'a> WriteCommandsConcurrent<'a> {
    pub fn new(
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        location: LocationIndexIteratorGetter<'a>,
    ) -> Self {
        Self {
            current_write,
            history_write,
            cache,
            file_dir,
            location,
        }
    }

    pub async fn save_to_tmp(
        &self,
        id: AccountIdInternal,
        stream: BodyStream,
    ) -> Result<ContentId, DatabaseError> {
        let content_id = ContentId::new_random_id();

        // Clear tmp dir if previous image writing failed and there is no
        // content ID in the database about it.
        self.file_dir
            .tmp_dir(id.as_light())
            .remove_contents_if_exists()
            .await
            .change_context(DatabaseError::File)?;

        let raw_img = self
            .file_dir
            .unprocessed_image_upload(id.as_light(), content_id);
        raw_img
            .save_stream(stream)
            .await
            .change_context(DatabaseError::File)?;

        // TODO: image safety checks and processing

        Ok(content_id)
    }

    pub async fn next_profiles(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<ProfileLink>, DatabaseError> {
        let location = self
            .cache
            .read_cache(id.as_light(), |e| {
                e.profile.as_ref().map(|p| p.location.clone())
            })
            .await
            .convert(id)?
            .ok_or(DatabaseError::FeatureDisabled)?;

        let iterator = self.location.get().ok_or(DatabaseError::FeatureDisabled)?;
        let (next_state, profiles) = iterator.next_profiles(location.current_iterator).await;
        self.cache
            .write_cache(id.as_light(), |e| {
                e.profile
                    .as_mut()
                    .map(move |p| p.location.current_iterator = next_state);
                Ok(())
            })
            .await
            .convert(id)?;

        Ok(profiles.unwrap_or(Vec::new()))
    }

    pub async fn reset_profile_iterator(&self, id: AccountIdInternal) -> Result<(), DatabaseError> {
        let location = self
            .cache
            .read_cache(id.as_light(), |e| {
                e.profile.as_ref().map(|p| p.location.clone())
            })
            .await
            .convert(id)?
            .ok_or(DatabaseError::FeatureDisabled)?;

        let iterator = self.location.get().ok_or(DatabaseError::FeatureDisabled)?;
        let next_state =
            iterator.reset_iterator(location.current_iterator, location.current_position);
        self.cache
            .write_cache(id.as_light(), |e| {
                e.profile
                    .as_mut()
                    .map(move |p| p.location.current_iterator = next_state);
                Ok(())
            })
            .await
            .convert(id)?;
        Ok(())
    }

    fn current(&self) -> CurrentDataWriteCommands {
        CurrentDataWriteCommands::new(&self.current_write)
    }

    fn history(&self) -> HistoryWriteCommands {
        HistoryWriteCommands::new(&self.history_write)
    }
}
