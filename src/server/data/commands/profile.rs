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
pub enum ProfileWriteCommand {
    UpdateProfile {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        profile: ProfileUpdateInternal,
    },
    UpdateProfileVisiblity {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        public: bool,
        update_only_if_none: bool,
    },
    UpdateProfileLocation {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        location: Location,
    },
}

#[derive(Debug, Clone)]
pub struct ProfileWriteCommandRunnerHandle<'a> {
    pub handle: &'a WriteCommandRunnerHandle,
}

impl ProfileWriteCommandRunnerHandle<'_> {
    pub async fn update_profile(
        &self,
        account_id: AccountIdInternal,
        profile: ProfileUpdateInternal,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| ProfileWriteCommand::UpdateProfile {
            s,
            account_id,
            profile,
        })
        .await
    }

    pub async fn update_profile_visiblity(
        &self,
        account_id: AccountIdInternal,
        public: bool,
        update_only_if_none: bool,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| ProfileWriteCommand::UpdateProfileVisiblity {
            s,
            account_id,
            public,
            update_only_if_none,
        })
        .await
    }

    pub async fn update_profile_location(
        &self,
        account_id: AccountIdInternal,
        location: Location,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| ProfileWriteCommand::UpdateProfileLocation {
            s,
            account_id,
            location,
        })
        .await
    }
}


impl WriteCommandRunner {
    pub async fn handle_profile_cmd(&self, cmd: ProfileWriteCommand) {
        match cmd {
            ProfileWriteCommand::UpdateProfile {
                s,
                account_id,
                profile,
            } => self.write().update_data(account_id, &profile).await.send(s),
            ProfileWriteCommand::UpdateProfileVisiblity {
                s,
                account_id,
                public,
                update_only_if_none,
            } => self
                .write()
                .profile_update_visibility(account_id, public, update_only_if_none)
                .await
                .send(s),
            ProfileWriteCommand::UpdateProfileLocation {
                s,
                account_id,
                location,
            } => self
                .write()
                .profile_update_location(account_id, location)
                .await
                .send(s),
        }
    }
}
