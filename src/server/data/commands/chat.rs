use super::{ResultSender, WriteCommandRunner, WriteCommandRunnerHandle};

use error_stack::Result;

use crate::{
    api::model::{AccountIdInternal, ModerationRequestContent},
    server::data::DatabaseError,
};

/// Synchronized write commands.
#[derive(Debug)]
pub enum ChatWriteCommand {
    Todo {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
    },
}

#[derive(Debug, Clone)]
pub struct ChatWriteCommandRunnerHandle<'a> {
    pub handle: &'a WriteCommandRunnerHandle,
}

impl ChatWriteCommandRunnerHandle<'_> {
    pub async fn set_moderation_request(
        &self,
        account_id: AccountIdInternal,
        _request: ModerationRequestContent,
    ) -> Result<(), DatabaseError> {
        self.handle
            .send_event(|s| ChatWriteCommand::Todo { s, account_id })
            .await
    }
}

impl WriteCommandRunner {
    pub async fn handle_chat_cmd(&self, cmd: ChatWriteCommand) {
        match cmd {
            ChatWriteCommand::Todo {
                s: _,
                account_id: _,
            } => unimplemented!(),
        }
    }
}
