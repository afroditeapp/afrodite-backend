use std::time::Duration;

use error_stack::{Result, ResultExt};
use manager_model::{BackupMessageType, SourceToTargetMessage, TargetToSourceMessage};
use simple_backend_utils::{ContextExt, IntoReportFromString};

use crate::{
    ClientError,
    protocol::{
        ClientConnectionReadSend, ClientConnectionWriteSend, ConnectionUtilsRead,
        ConnectionUtilsWrite,
    },
};

pub struct BackupSourceClient {
    reader: Box<dyn ClientConnectionReadSend>,
    writer: Box<dyn ClientConnectionWriteSend>,
    backup_session: u32,
}

impl BackupSourceClient {
    pub fn new(
        reader: Box<dyn ClientConnectionReadSend>,
        writer: Box<dyn ClientConnectionWriteSend>,
        backup_session: u32,
    ) -> Self {
        Self {
            reader,
            writer,
            backup_session,
        }
    }

    pub async fn send_message(
        &mut self,
        message: SourceToTargetMessage,
    ) -> Result<(), ClientError> {
        let m = message
            .into_message(self.backup_session)
            .into_error_string(ClientError::Parse)?;
        self.writer
            .send_backup_link_message(m)
            .await
            .change_context(ClientError::Write)
    }

    /// 30 second timeout
    pub async fn receive_message(&mut self) -> Result<TargetToSourceMessage, ClientError> {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(30)) =>
                Err(ClientError::Timeout.report()),
            r = self.receive_message_no_timeout() => r,
        }
    }

    async fn receive_message_no_timeout(&mut self) -> Result<TargetToSourceMessage, ClientError> {
        loop {
            let Some(m) = self
                .reader
                .receive_backup_link_message()
                .await
                .change_context(ClientError::Read)?
            else {
                return Err(ClientError::Read.report());
            };

            if m.header.message_type == BackupMessageType::Empty {
                continue;
            }

            if m.header.backup_session.0 != self.backup_session {
                continue;
            }

            let m = m.try_into().into_error_string(ClientError::Parse)?;

            return Ok(m);
        }
    }
}
