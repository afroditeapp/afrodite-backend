use database::{current::read::CurrentSyncReadCommands, CurrentReadHandle, DieselConnection, DieselDatabaseError};
use error_stack::ResultExt;
use model::{
    AccountId, AccountIdInternal, ContentId, MediaContentRaw, ModerationRequest,
    ModerationRequestState,
};
use server_common::data::DataError;
use simple_backend_utils::IntoReportFromString;
use tokio_util::io::ReaderStream;

use self::{
    account::ReadCommandsAccount, account_admin::ReadCommandsAccountAdmin, chat::ReadCommandsChat,
    chat_admin::ReadCommandsChatAdmin, common::ReadCommandsCommon, media::ReadCommandsMedia,
    media_admin::ReadCommandsMediaAdmin, profile::ReadCommandsProfile,
    profile_admin::ReadCommandsProfileAdmin,
};
use super::{cache::DatabaseCache, file::utils::FileDir, IntoDataError};
use crate::result::Result;

macro_rules! define_read_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: crate::read::ReadCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: crate::read::ReadCommands<'a>) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn cache(&self) -> &crate::DatabaseCache {
                &self.cmds.cache
            }

            #[allow(dead_code)]
            fn files(&self) -> &crate::FileDir {
                &self.cmds.files
            }

            pub async fn db_read<
                T: FnOnce(
                        database::current::read::CurrentSyncReadCommands<
                            &mut database::DieselConnection,
                        >,
                    ) -> error_stack::Result<
                        R,
                        database::DieselDatabaseError,
                    > + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            {
                self.cmds.db_read(cmd).await
            }

            // TODO: change cache operation to return Result?
            pub async fn read_cache<T, Id: Into<model::AccountId>>(
                &self,
                id: Id,
                cache_operation: impl Fn(&crate::cache::CacheEntry) -> T,
            ) -> error_stack::Result<T, crate::CacheError> {
                self.cache().read_cache(id, cache_operation).await
            }
        }
    };
}

pub mod account;
pub mod account_admin;
pub mod chat;
pub mod chat_admin;
pub mod common;
pub mod media;
pub mod media_admin;
pub mod profile;
pub mod profile_admin;

pub struct ReadCommands<'a> {
    db: &'a CurrentReadHandle,
    cache: &'a DatabaseCache,
    files: &'a FileDir,
}

impl<'a> ReadCommands<'a> {
    pub fn new(
        current_read_handle: &'a CurrentReadHandle,
        cache: &'a DatabaseCache,
        files: &'a FileDir,
    ) -> Self {
        Self {
            db: current_read_handle,
            cache,
            files,
        }
    }

    pub fn account(self) -> ReadCommandsAccount<'a> {
        ReadCommandsAccount::new(self)
    }

    pub fn account_admin(self) -> ReadCommandsAccountAdmin<'a> {
        ReadCommandsAccountAdmin::new(self)
    }

    pub fn media(self) -> ReadCommandsMedia<'a> {
        ReadCommandsMedia::new(self)
    }

    pub fn media_admin(self) -> ReadCommandsMediaAdmin<'a> {
        ReadCommandsMediaAdmin::new(self)
    }

    pub fn profile(self) -> ReadCommandsProfile<'a> {
        ReadCommandsProfile::new(self)
    }

    pub fn profile_admin(self) -> ReadCommandsProfileAdmin<'a> {
        ReadCommandsProfileAdmin::new(self)
    }

    pub fn chat(self) -> ReadCommandsChat<'a> {
        ReadCommandsChat::new(self)
    }

    pub fn chat_admin(self) -> ReadCommandsChatAdmin<'a> {
        ReadCommandsChatAdmin::new(self)
    }

    pub fn common(self) -> ReadCommandsCommon<'a> {
        ReadCommandsCommon::new(self)
    }

    pub async fn image_stream(
        &self,
        account_id: AccountId,
        content_id: ContentId,
    ) -> Result<ReaderStream<tokio::fs::File>, DataError> {
        self.files
            .media_content(account_id, content_id)
            .read_stream()
            .await
            .into_data_error((account_id, content_id))
    }

    pub async fn all_account_media_content(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<MediaContentRaw>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media()
                .media_content()
                .get_account_media_content(account_id)
        })
        .await
        .into_error()
    }

    pub async fn moderation_request(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Option<ModerationRequest>, DataError> {
        let request = self
            .db_read(move |mut cmds| {
                cmds.media()
                    .moderation_request()
                    .moderation_request(account_id)
            })
            .await
            .into_error()?;

        if let Some(request) = request {
            let smallest_number = self
                .db_read(move |mut cmds| {
                    cmds.common()
                        .queue_number()
                        .smallest_queue_number(request.queue_type)
                })
                .await?;
            let num = if let (Some(num), true) = (
                smallest_number,
                request.state == ModerationRequestState::Waiting,
            ) {
                let queue_position = request.queue_number.0 - num;
                Some(i64::max(queue_position, 0))
            } else {
                None
            };
            Ok(Some(ModerationRequest::new(request, num)))
        } else {
            Ok(None)
        }
    }

    pub async fn db_read<
        T: FnOnce(
                CurrentSyncReadCommands<&mut DieselConnection>,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReader { db: self.db }.db_read(cmd).await
    }
}

pub struct DbReader<'a> {
    db: &'a CurrentReadHandle,
}

impl<'a> DbReader<'a> {
    pub fn new(db: &'a CurrentReadHandle) -> Self {
        Self { db }
    }

    pub async fn db_read<
        T: FnOnce(
                CurrentSyncReadCommands<&mut DieselConnection>,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        let conn = self
            .db
            .0
            .diesel()
            .pool()
            .get()
            .await
            .change_context(DieselDatabaseError::GetConnection)?;

        conn.interact(move |conn| cmd(CurrentSyncReadCommands::new(conn)))
            .await
            .into_error_string(DieselDatabaseError::Execute)?
    }
}
