use error_stack::{Result, ResultExt, report};
use manager_model::{
    BackupMessage, BackupMessageHeader, BackupMessageType, JsonRpcLinkHeader, JsonRpcLinkMessage,
    JsonRpcLinkMessageType, JsonRpcRequest, JsonRpcRequestType, JsonRpcResponse,
    JsonRpcResponseType, ManagerInstanceName, ManagerInstanceNameList, ManagerProtocolMode,
    ManagerProtocolVersion, ManualTaskType, NotifyBackend, ScheduledTaskStatus, ScheduledTaskType,
    SecureStorageEncryptionKey, ServerEvent, SoftwareUpdateStatus, SoftwareUpdateTaskType,
    SystemInfo,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{ClientError, ManagerClient};

pub trait ClientConnectionReadWrite: ClientConnectionRead + ClientConnectionWrite {}
impl<T: ClientConnectionRead + ClientConnectionWrite> ClientConnectionReadWrite for T {}

pub trait ClientConnectionRead: tokio::io::AsyncRead + std::marker::Unpin {}
impl<T: tokio::io::AsyncRead + std::marker::Unpin> ClientConnectionRead for T {}

pub trait ClientConnectionWrite: tokio::io::AsyncWrite + std::marker::Unpin {}
impl<T: tokio::io::AsyncWrite + std::marker::Unpin> ClientConnectionWrite for T {}

pub trait ClientConnectionReadWriteSend:
    ClientConnectionReadSend + ClientConnectionWriteSend
{
}
impl<T: ClientConnectionReadSend + ClientConnectionWriteSend> ClientConnectionReadWriteSend for T {}

pub trait ClientConnectionReadSend: ClientConnectionRead + Send + 'static {}
impl<T: ClientConnectionRead + Send + 'static> ClientConnectionReadSend for T {}

pub trait ClientConnectionWriteSend: ClientConnectionWrite + Send + 'static {}
impl<T: ClientConnectionWrite + Send + 'static> ClientConnectionWriteSend for T {}

pub trait ConnectionUtilsRead: tokio::io::AsyncRead + Unpin {
    async fn receive_u8_optional(&mut self) -> Result<Option<u8>, ClientError> {
        let mut buf = [0u8];
        let size = self
            .read(&mut buf)
            .await
            .change_context(ClientError::Read)?;
        if size == 0 {
            Ok(None)
        } else if size == 1 {
            Ok(Some(buf[0]))
        } else {
            Err(report!(ClientError::Read))
                .attach_printable(format!("Unknown reading result size {}", size))
        }
    }

    async fn receive_u8(&mut self) -> Result<u8, ClientError> {
        self.read_u8().await.change_context(ClientError::Read)
    }

    async fn receive_vec_with_u32_len(&mut self) -> Result<Vec<u8>, ClientError> {
        let len = self.read_u32_le().await.change_context(ClientError::Read)?;
        let len_usize: usize =
            TryInto::<usize>::try_into(len).change_context(ClientError::UnsupportedDataSize)?;
        let mut vec: Vec<u8> = vec![0; len_usize];
        self.read_exact(&mut vec)
            .await
            .change_context(ClientError::Read)?;
        Ok(vec)
    }

    async fn receive_string_with_u32_len(&mut self) -> Result<String, ClientError> {
        let len = self.read_u32_le().await.change_context(ClientError::Read)?;
        let len_usize: usize =
            TryInto::<usize>::try_into(len).change_context(ClientError::UnsupportedStringLength)?;
        let mut vec: Vec<u8> = vec![0; len_usize];
        self.read_exact(&mut vec)
            .await
            .change_context(ClientError::Read)?;
        String::from_utf8(vec).change_context(ClientError::Parse)
    }

    async fn receive_protocol_version(&mut self) -> Result<ManagerProtocolVersion, ClientError> {
        TryInto::<ManagerProtocolVersion>::try_into(self.receive_u8().await?)
            .change_context(ClientError::Parse)
    }

    async fn receive_protocol_mode(&mut self) -> Result<ManagerProtocolMode, ClientError> {
        TryInto::<ManagerProtocolMode>::try_into(self.receive_u8().await?)
            .change_context(ClientError::Parse)
    }

    async fn receive_json_rpc_request(&mut self) -> Result<JsonRpcRequest, ClientError> {
        let s = self
            .receive_string_with_u32_len()
            .await
            .change_context(ClientError::Read)?;
        serde_json::from_str(&s).change_context(ClientError::Parse)
    }

    async fn receive_json_rpc_response(&mut self) -> Result<JsonRpcResponse, ClientError> {
        let s = self
            .receive_string_with_u32_len()
            .await
            .change_context(ClientError::Read)?;
        serde_json::from_str(&s).change_context(ClientError::Parse)
    }

    async fn receive_server_event(&mut self) -> Result<ServerEvent, ClientError> {
        let s = self
            .receive_string_with_u32_len()
            .await
            .change_context(ClientError::Read)?;
        serde_json::from_str(&s).change_context(ClientError::Parse)
    }

    /// If None, connection is disconnected
    async fn receive_json_rpc_link_message(
        &mut self,
    ) -> Result<Option<JsonRpcLinkMessage>, ClientError> {
        let Some(message_type) = self.receive_u8_optional().await? else {
            return Ok(None);
        };
        let message_type = TryInto::<JsonRpcLinkMessageType>::try_into(message_type)
            .change_context(ClientError::Parse)?;
        let sequence_number = self.read_u32_le().await.change_context(ClientError::Read)?;
        let data = self
            .receive_string_with_u32_len()
            .await
            .change_context(ClientError::Read)?;

        Ok(Some(JsonRpcLinkMessage {
            header: JsonRpcLinkHeader {
                sequence_number: std::num::Wrapping(sequence_number),
                message_type,
            },
            data,
        }))
    }

    /// If None, connection is disconnected
    async fn receive_backup_link_message(&mut self) -> Result<Option<BackupMessage>, ClientError> {
        let Some(message_type) = self.receive_u8_optional().await? else {
            return Ok(None);
        };
        let message_type = TryInto::<BackupMessageType>::try_into(message_type)
            .change_context(ClientError::Parse)?;
        let backup_session = self.read_u32_le().await.change_context(ClientError::Read)?;
        let data = self
            .receive_vec_with_u32_len()
            .await
            .change_context(ClientError::Read)?;

        Ok(Some(BackupMessage {
            header: BackupMessageHeader {
                backup_session: std::num::Wrapping(backup_session),
                message_type,
            },
            data,
        }))
    }
}

impl<T: tokio::io::AsyncRead + Unpin> ConnectionUtilsRead for T {}

pub trait ConnectionUtilsWrite: tokio::io::AsyncWrite + Unpin {
    async fn send_u8(&mut self, byte: u8) -> Result<(), ClientError> {
        self.write_u8(byte)
            .await
            .change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }

    async fn send_vec_with_u32_len(&mut self, data: Vec<u8>) -> Result<(), ClientError> {
        let len_u32: u32 = TryInto::<u32>::try_into(data.len())
            .change_context(ClientError::UnsupportedDataSize)?;
        self.write_u32_le(len_u32)
            .await
            .change_context(ClientError::Write)?;
        self.write_all(&data)
            .await
            .change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }

    async fn send_string_with_u32_len(&mut self, value: String) -> Result<(), ClientError> {
        let len_u32: u32 = TryInto::<u32>::try_into(value.len())
            .change_context(ClientError::UnsupportedStringLength)?;
        self.write_u32_le(len_u32)
            .await
            .change_context(ClientError::Write)?;
        self.write_all(value.as_bytes())
            .await
            .change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }

    async fn send_json_rpc_request(&mut self, request: JsonRpcRequest) -> Result<(), ClientError> {
        let text = serde_json::to_string(&request).change_context(ClientError::Serialize)?;
        self.send_string_with_u32_len(text)
            .await
            .change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }

    async fn send_json_rpc_response(
        &mut self,
        response: JsonRpcResponse,
    ) -> Result<(), ClientError> {
        let text = serde_json::to_string(&response).change_context(ClientError::Serialize)?;
        self.send_string_with_u32_len(text)
            .await
            .change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }

    async fn send_server_event(&mut self, server_event: &ServerEvent) -> Result<(), ClientError> {
        let text = serde_json::to_string(server_event).change_context(ClientError::Serialize)?;
        self.send_string_with_u32_len(text)
            .await
            .change_context(ClientError::Write)?;
        self.flush().await.change_context(ClientError::Flush)?;
        Ok(())
    }

    async fn send_json_rpc_link_message(
        &mut self,
        message: JsonRpcLinkMessage,
    ) -> Result<(), ClientError> {
        self.send_u8(message.header.message_type as u8)
            .await
            .change_context(ClientError::Write)?;
        self.write_u32_le(message.header.sequence_number.0)
            .await
            .change_context(ClientError::Write)?;
        self.send_string_with_u32_len(message.data)
            .await
            .change_context(ClientError::Write)?;

        self.flush().await.change_context(ClientError::Flush)?;

        Ok(())
    }

    async fn send_backup_link_message(
        &mut self,
        message: BackupMessage,
    ) -> Result<(), ClientError> {
        self.send_u8(message.header.message_type as u8)
            .await
            .change_context(ClientError::Write)?;
        self.write_u32_le(message.header.backup_session.0)
            .await
            .change_context(ClientError::Write)?;
        self.send_vec_with_u32_len(message.data)
            .await
            .change_context(ClientError::Write)?;

        self.flush().await.change_context(ClientError::Flush)?;

        Ok(())
    }
}

impl<T: tokio::io::AsyncWrite + Unpin> ConnectionUtilsWrite for T {}

pub struct ManagerClientWithRequestReceiver {
    pub(crate) client: ManagerClient,
    pub(crate) request_receiver: ManagerInstanceName,
}

pub trait RequestSenderCmds: Sized {
    fn request_receiver_name(&self) -> ManagerInstanceName;
    async fn send_request(self, request: JsonRpcRequest) -> Result<JsonRpcResponse, ClientError>;

    async fn get_available_instances(self) -> Result<ManagerInstanceNameList, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetManagerInstanceNames,
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::ManagerInstanceNames(info) = response.into_response() {
            Ok(info)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn get_secure_storage_encryption_key(
        self,
        key: ManagerInstanceName,
    ) -> Result<SecureStorageEncryptionKey, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetSecureStorageEncryptionKey(key),
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::SecureStorageEncryptionKey(key) = response.into_response() {
            Ok(key)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn get_system_info(self) -> Result<SystemInfo, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetSystemInfo,
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::SystemInfo(info) = response.into_response() {
            Ok(info)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn get_software_update_status(self) -> Result<SoftwareUpdateStatus, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetSoftwareUpdateStatus,
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::SoftwareUpdateStatus(status) = response.into_response() {
            Ok(status)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn trigger_software_update_task(
        self,
        task: SoftwareUpdateTaskType,
    ) -> Result<(), ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::TriggerSoftwareUpdateTask(task),
        );
        self.send_request(request).await?.require_successful()
    }

    async fn trigger_manual_task(self, task: ManualTaskType) -> Result<(), ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::TriggerManualTask(task),
        );
        self.send_request(request).await?.require_successful()
    }

    async fn get_scheduled_tasks_status(self) -> Result<ScheduledTaskStatus, ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::GetScheduledTasksStatus,
        );
        let response = self.send_request(request).await?;
        if let JsonRpcResponseType::ScheduledTasksStatus(status) = response.into_response() {
            Ok(status)
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }

    async fn schedule_task(
        self,
        task: ScheduledTaskType,
        notify_backend: NotifyBackend,
    ) -> Result<(), ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::ScheduleTask(task, notify_backend),
        );
        self.send_request(request).await?.require_successful()
    }

    async fn unschedule_task(self, task: ScheduledTaskType) -> Result<(), ClientError> {
        let request = JsonRpcRequest::new(
            self.request_receiver_name(),
            JsonRpcRequestType::UnscheduleTask(task),
        );
        self.send_request(request).await?.require_successful()
    }
}

trait RpcResponseExtensions: Sized {
    fn require_successful(self) -> Result<(), ClientError>;
}

impl RpcResponseExtensions for JsonRpcResponse {
    fn require_successful(self) -> Result<(), ClientError> {
        if let JsonRpcResponseType::Successful = self.into_response() {
            Ok(())
        } else {
            Err(report!(ClientError::InvalidResponse))
        }
    }
}

impl RequestSenderCmds for ManagerClientWithRequestReceiver {
    fn request_receiver_name(&self) -> ManagerInstanceName {
        self.request_receiver.clone()
    }
    async fn send_request(self, request: JsonRpcRequest) -> Result<JsonRpcResponse, ClientError> {
        self.client.send_request(request).await
    }
}
