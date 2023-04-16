//! Database writing commands
//!

// TODO: Remove this module at some point, if use case for it does not appear.

use std::{sync::Arc, future::Future};

use api_client::models::AccountId;
use axum::extract::BodyStream;
use error_stack::{Result, ResultExt, Report};
use serde::Serialize;
use tokio::sync::{MutexGuard, Mutex};
use tokio_stream::StreamExt;

use crate::{
    api::{model::{Account, AccountIdInternal, AccountSetup, ApiKey, Profile, AccountIdLight, ContentId, NewModerationRequest}, media::data::Moderation},
    config::Config,
    server::database::{sqlite::SqliteWriteHandle, DatabaseError},
    utils::{ErrorConversion, IntoReportExt, AppendErrorTo},
};

use super::{
    current::write::CurrentDataWriteCommands,
    history::write::HistoryWriteCommands,
    sqlite::{HistoryUpdateJson, SqliteUpdateJson, CurrentDataWriteHandle, HistoryWriteHandle},
    utils::GetReadWriteCmd, cache::{DatabaseCache, WriteCacheJson}, file::{utils::{ FileDir}, file::ImageSlot}, RouterDatabaseHandle,
};


use tokio::sync::oneshot::{Sender, Receiver};

pub type ResultSender<T> = Sender<Result<T, DatabaseError>>;

/// Synchronized write commands.
#[derive(Debug)]
pub enum WriteCommand {
    SetNewApiKey{ result_sender: ResultSender<()>, account_id: AccountIdInternal, data: ApiKey },
    UpdateAccount{ result_sender: ResultSender<()>, account_id: AccountIdInternal, data: Account },
    UpdateProfile{ result_sender: ResultSender<()>, account_id: AccountIdInternal, data: Profile },
    SetModerationRequest{ result_sender: ResultSender<()>, account_id: AccountIdInternal, data: NewModerationRequest },
    GetModerationListAndCreateNewIfNecessary { result_sender: ResultSender<Vec<Moderation>>, account_id: AccountIdInternal },
}


/// Concurrent write commands.
#[derive(Debug)]
pub enum ConcurrentWriteCommand {
    SaveToSlot { result_sender: ResultSender<ContentId>, account_id: AccountIdInternal },
}



#[derive(Debug)]
pub struct WriteCommandRunnerQuitHandle {

}

impl WriteCommandRunnerHandle {
    pub async fn quit(self) {

    }
}

#[derive(Debug)]
pub struct WriteCommandRunnerHandle {

}


#[derive(Debug)]
pub struct WriteCommandRunner {

}


impl WriteCommandRunner {

    pub async fn run(self) {

    }
}
