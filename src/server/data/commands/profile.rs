use super::{ResultSender, SendBack, WriteCommandRunner, WriteCommandRunnerHandle};

use error_stack::Result;

use crate::{
    api::model::{AccountIdInternal, Location, ProfileUpdateInternal},
    server::data::DatabaseError,
};

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
        self.handle
            .send_event(|s| ProfileWriteCommand::UpdateProfile {
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
        self.handle
            .send_event(|s| ProfileWriteCommand::UpdateProfileVisiblity {
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
        self.handle
            .send_event(|s| ProfileWriteCommand::UpdateProfileLocation {
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
                .profile()
                .profile_update_visibility(account_id, public, update_only_if_none)
                .await
                .send(s),
            ProfileWriteCommand::UpdateProfileLocation {
                s,
                account_id,
                location,
            } => self
                .write()
                .profile()
                .profile_update_location(account_id, location)
                .await
                .send(s),
        }
    }
}
