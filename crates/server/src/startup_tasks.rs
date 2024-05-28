use server_common::{data::DataError, result::Result};

use crate::app::{GetConfig, WriteData, S};
pub struct StartupTasks {
    state: S,
}

impl StartupTasks {
    pub fn new(state: S) -> Self {
        Self { state }
    }

    pub async fn run_and_wait_completion(self) -> Result<(), DataError> {
        Self::handle_profile_attribute_file_changes(&self.state).await
    }

    async fn handle_profile_attribute_file_changes(state: &S) -> Result<(), DataError> {
        let hash = if let Some(hash) = state.config().profile_attributes_sha256() {
            hash.to_string()
        } else {
            return Ok(());
        };

        state
            .write(move |cmds| async move {
                cmds.profile()
                    .update_profile_attributes_sha256_and_sync_versions(hash)
                    .await
            })
            .await
    }
}
