//! CLI API client
//!

use error_stack::{Result, ResultExt};
use manager_model::{ManagerInstanceName, SoftwareInfo, SoftwareUpdateTaskType};

use manager_config::args::{ApiCommand, ManagerApiClientMode};
use manager_api::{ClientConfig, ClientError, ManagerClient, protocol::RequestSenderCmds};

pub async fn handle_api_client_mode(args: ManagerApiClientMode) -> Result<(), ClientError> {
    let api_key = args
        .api_key()
        .change_context(ClientError::MissingConfiguration)?;
    let api_url = args
        .api_url()
        .change_context(ClientError::MissingConfiguration)?;
    let tls_config = args
        .tls_config()
        .change_context(ClientError::InvalidConfiguration)?;
    let manager_name = args
        .manager_name()
        .change_context(ClientError::MissingConfiguration)?;

    let config = ClientConfig {
        api_key,
        url: api_url,
        tls_config,
    };

    let client = ManagerClient::connect(config)
        .await
        .change_context(ClientError::RemoteApiRequest)?
        .request_to(manager_name);

    match args.api_command {
        ApiCommand::AvailableInstances => {
            let list = client.get_available_instances()
                .await
                .change_context(ClientError::RemoteApiRequest)?;
            println!("{:#?}", list);
        }
        ApiCommand::EncryptionKey {
            encryption_key_name,
        } => {
            let key = client.get_secure_storage_encryption_key(
                ManagerInstanceName::new(encryption_key_name.clone()),
            )
                .await
                .change_context(ClientError::RemoteApiRequest)?;
            println!("Name: {}", encryption_key_name);
            println!("Key:  {}", key.key);
        }
        ApiCommand::SystemInfo => {
            let info = client.get_system_info()
                .await
                .change_context(ClientError::RemoteApiRequest)?;
            println!("{:#?}", info);
        }
        ApiCommand::SoftwareStatus => {
            let info = client.get_software_update_status()
                .await
                .change_context(ClientError::RemoteApiRequest)?;
            println!("{:#?}", info);
        }
        ApiCommand::SoftwareDownload => {
            client.trigger_software_update_task(SoftwareUpdateTaskType::Download)
                .await
                .change_context(ClientError::RemoteApiRequest)?
        }
        ApiCommand::SoftwareInstall { name, sha256 } => {
            client.trigger_software_update_task(SoftwareUpdateTaskType::Install(SoftwareInfo { name, sha256 }))
                .await
                .change_context(ClientError::RemoteApiRequest)?
        }
    }

    Ok(())
}
