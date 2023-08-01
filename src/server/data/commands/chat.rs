use super::{WriteCommandRunnerHandle, ResultSender, WriteCommandRunner, SendBack};

use std::{collections::HashSet, future::Future, net::SocketAddr, sync::Arc};

use axum::extract::BodyStream;
use error_stack::Result;

use tokio::{
    sync::{mpsc, oneshot, OwnedSemaphorePermit, RwLock, Semaphore},
    task::JoinHandle,
};
use tokio_stream::StreamExt;

use crate::{
    api::{
        media::data::{HandleModerationRequest, Moderation},
        model::{
            Account, AccountIdInternal, AccountIdLight, AccountSetup, AuthPair, ContentId,
            Location, ModerationRequestContent, ProfileLink,
            ProfileUpdateInternal, SignInWithInfo,
        },
    },
    config::Config,
    server::data::{write::WriteCommands, DatabaseError},
    utils::{ErrorConversion, IntoReportExt},
};

use super::{super::file::file::ImageSlot, RouterDatabaseWriteHandle};



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
        request: ModerationRequestContent,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| ChatWriteCommand::Todo {
            s,
            account_id,
        })
        .await
    }
}

impl WriteCommandRunner {
    pub async fn handle_chat_cmd(&self, cmd: ChatWriteCommand) {
        match cmd {
            ChatWriteCommand::Todo {
                s,
                account_id,
            } => unimplemented!()
        }
    }
}
