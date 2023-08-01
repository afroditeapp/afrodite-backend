use super::{
    super::{cache::DatabaseCache, file::utils::FileDir},
    ReadCommands,
};

use crate::server::data::database::current::SqliteReadCommands;

define_read_commands!(ReadCommandsChat);

impl ReadCommandsChat<'_> {}
