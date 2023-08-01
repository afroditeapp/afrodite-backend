use std::{fmt::Debug, marker::PhantomData};

use serde_json::de::Read;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

use crate::{
    api::{
        media::data::{ModerationRequest, PrimaryImage, CurrentAccountMediaInternal, MediaContentType, MediaContentInternal},
        model::{
            AccountIdInternal, AccountIdLight, ApiKey, ContentId, RefreshToken, SignInWithInfo,
        },
    },
    utils::{ConvertCommandError, ErrorConversion},
};

use super::{super::{
    cache::{CacheError, DatabaseCache, ReadCacheJson},
    current::SqliteReadCommands,
    file::{utils::FileDir, FileError},
    sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson},
    write::NoId,
    DatabaseError,
}, ReadCommands};

use error_stack::Result;

define_read_commands!(ReadCommandsAccountAdmin);

impl ReadCommandsAccountAdmin<'_> {

}
