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

define_read_commands!(ReadCommandsProfile);

impl ReadCommandsProfile<'_> {

}
