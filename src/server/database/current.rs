pub mod account;
pub mod media;
pub mod profile;
pub mod chat;

use self::chat::write::CurrentWriteChatCommands;
use self::{account::read::CurrentReadAccountCommands, chat::read::CurrentReadChatCommands};
use self::account::write::CurrentWriteAccountCommands;
use self::media::admin_write::CurrentWriteMediaAdminCommands;
use self::media::read::CurrentReadMediaCommands;
use self::media::write::CurrentWriteMediaCommands;
use self::profile::read::CurrentReadProfileCommands;
use self::profile::write::CurrentWriteProfileCommands;

use super::sqlite::CurrentDataWriteHandle;

use crate::server::database::sqlite::{SqliteReadHandle};

#[macro_export]
macro_rules! read_json {
    ($self:expr, $id:expr, $sql:literal, $str_field:ident) => {{
        let id = $id.row_id();
        sqlx::query!($sql, id)
            .fetch_one($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)
            .and_then(|data| {
                serde_json::from_str(&data.$str_field)
                    .into_error(SqliteDatabaseError::SerdeDeserialize)
            })
    }};
}

#[macro_export]
macro_rules! insert_or_update_json {
    ($self:expr, $sql:literal, $data:expr, $id:expr) => {{
        let id = $id.row_id();
        let data = serde_json::to_string($data).into_error(SqliteDatabaseError::SerdeSerialize)?;
        sqlx::query!($sql, data, id)
            .execute($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }};
}

pub struct SqliteReadCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> SqliteReadCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }

    pub fn account(&self) -> CurrentReadAccountCommands<'_> {
        CurrentReadAccountCommands::new(self.handle)
    }

    pub fn media(&self) -> CurrentReadMediaCommands<'_> {
        CurrentReadMediaCommands::new(self.handle)
    }

    pub fn profile(&self) -> CurrentReadProfileCommands<'_> {
        CurrentReadProfileCommands::new(self.handle)
    }

    pub fn chat(&self) -> CurrentReadChatCommands<'_> {
        CurrentReadChatCommands::new(self.handle)
    }
}

#[derive(Clone, Debug)]
pub struct CurrentDataWriteCommands<'a> {
    handle: &'a CurrentDataWriteHandle,
}

impl<'a> CurrentDataWriteCommands<'a> {
    pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
        Self { handle }
    }

    pub fn account(self) -> CurrentWriteAccountCommands<'a> {
        CurrentWriteAccountCommands::new(self.handle)
    }

    pub fn media(self) -> CurrentWriteMediaCommands<'a> {
        CurrentWriteMediaCommands::new(self.handle)
    }

    pub fn media_admin(self) -> CurrentWriteMediaAdminCommands<'a> {
        CurrentWriteMediaAdminCommands::new(self.handle)
    }

    pub fn profile(self) -> CurrentWriteProfileCommands<'a> {
        CurrentWriteProfileCommands::new(self.handle)
    }

    pub fn chat(self) -> CurrentWriteChatCommands<'a> {
        CurrentWriteChatCommands::new(self.handle)
    }
}
