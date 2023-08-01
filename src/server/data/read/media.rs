use std::{fmt::Debug, marker::PhantomData};

use serde_json::de::Read;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

use crate::{
    api::{
        media::data::{CurrentAccountMediaInternal, MediaContentInternal, MediaContentType, ModerationRequest, PrimaryImage},
        model::{
            AccountIdInternal, AccountIdLight, ApiKey, ContentId, RefreshToken, SignInWithInfo,
        },
    },
    utils::{ConvertCommandError, ErrorConversion},
};

use super::{ReadCommands, super::{
    cache::{CacheError, DatabaseCache, ReadCacheJson},
    DatabaseError,
    file::{FileError, utils::FileDir},
    database::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson},
    write::NoId,
}};

use error_stack::Result;
use crate::server::data::database::current::SqliteReadCommands;


define_read_commands!(ReadCommandsMedia);

impl ReadCommandsMedia<'_> {
    pub async fn image(
        &self,
        account_id: AccountIdLight,
        content_id: ContentId,
    ) -> Result<Vec<u8>, DatabaseError> {
        self.files()
            .image_content(account_id, content_id)
            .read_all()
            .await
            .convert((account_id, content_id))
    }

    pub async fn current_account_media(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<CurrentAccountMediaInternal, DatabaseError> {
        self.db()
            .media()
            .get_current_account_media(account_id)
            .await
            .convert(account_id)
    }
}
