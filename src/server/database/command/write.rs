use std::{path::{Path, PathBuf}, fmt, thread::sleep, time::Duration};

use tokio::sync::oneshot;

use crate::{api::core::user::{RegisterBody, RegisterResponse, LoginBody, LoginResponse}, server::database::{util::{DatabasePath, ProfileDirPath}, DatabaseError, DatabaseOperationHandle, file::CoreFile}};

use super::{DatabeseEntryId, super::git::GitDatabase};



pub struct DatabaseBasicCommands<'a> {
    database: &'a DatabasePath,
}

impl <'a> DatabaseBasicCommands<'a> {
    pub fn new(database: &'a DatabasePath) -> Self {
        Self { database }
    }


}



/// Make sure that you do not make concurrent writes.
pub struct DatabaseWriteCommands {
    profile: ProfileDirPath,
    handle: DatabaseOperationHandle,
}

impl DatabaseWriteCommands {
    pub fn new(profile: ProfileDirPath, handle: DatabaseOperationHandle) -> Self {
        Self { profile, handle }
    }

    pub async fn register(self) -> Result<(), DatabaseError> {
        let task = tokio::task::spawn_blocking(|| self.register_blocking() );
        task.await.unwrap()
    }

    fn register_blocking(self) -> Result<(), DatabaseError> {
        GitDatabase::create(&self.profile).map_err(DatabaseError::Git)?;
        sleep(Duration::from_secs(5));
        Ok(())
        // let file = profile.create_file(CoreFile::ProfileJson);
        // TODO: Write something to the JSON

        // git.commit(CoreFile::ProfileJson, "Initial profile information")
        //     .map_err(DatabaseError::Git)
        //     .map(|_| profile)
    }


    // fn write_api_token_bloking(profile: ProfileDirPath, token: Option<UserApiToken>) -> LoginResponse {

    //     LoginResponse::database_error()
    // }
}
