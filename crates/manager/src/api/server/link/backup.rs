use std::net::SocketAddr;

use error_stack::{Result, ResultExt};
use manager_api::protocol::{
    ClientConnectionRead, ClientConnectionWrite, ConnectionUtilsRead, ConnectionUtilsWrite,
};
use tracing::info;

use super::super::{ClientConnectionReadWrite, ServerError};
use crate::{
    api::{
        GetBackupLinkManager,
        utils::{BackupLinkClient, validate_backup_link_login},
    },
    server::{app::S, link::backup::server::BackupLinkConnectionReceiver},
};

pub async fn handle_backup_link<C: ClientConnectionReadWrite>(
    mut c: C,
    address: SocketAddr,
    state: S,
) -> Result<(), ServerError> {
    let password = c
        .receive_string_with_u32_len()
        .await
        .change_context(ServerError::Read)?;

    let Ok(client) = validate_backup_link_login(&state, address, &password) else {
        c.send_u8(0).await.change_context(ServerError::Write)?;
        return Ok(());
    };

    c.send_u8(1).await.change_context(ServerError::Write)?;

    handle_server_messages(c, address, state, client).await
}

async fn handle_server_messages<C: ClientConnectionReadWrite>(
    c: C,
    address: SocketAddr,
    state: S,
    client: BackupLinkClient,
) -> Result<(), ServerError> {
    info!(
        "Backup client type {:?} from {} started backup link connection",
        client, address
    );

    let (reader, writer) = tokio::io::split(c);

    let mut receiver = match client {
        BackupLinkClient::Source => {
            let Some(r) = state
                .backup_link_server()
                .replace_source_connection()
                .await
                .change_context(ServerError::BackupLink)?
            else {
                info!(
                    "Backup client {:?} from {} can not connect because target client is not connected",
                    client, address
                );
                return Ok(());
            };

            r
        }
        BackupLinkClient::Target => state
            .backup_link_server()
            .replace_target_connection()
            .await
            .change_context(ServerError::BackupLink)?,
    };

    let r = tokio::select! {
        r = handle_read(reader, client, state.clone()) => {
            r
        }
        r = handle_write(writer, &mut receiver) => {
            r
        },
    };

    drop(receiver);
    state
        .backup_link_server()
        .clean_connection(client)
        .await
        .change_context(ServerError::BackupLink)?;

    info!(
        "Backup client type {:?} from {} disconnected backup link connection",
        client, address
    );

    r
}

async fn handle_read<C: ClientConnectionRead>(
    mut c: C,
    client: BackupLinkClient,
    state: S,
) -> Result<(), ServerError> {
    loop {
        let Some(connection) = handle_read_one_message(c, client, &state).await? else {
            return Ok(());
        };
        c = connection;
    }
}

async fn handle_read_one_message<C: ClientConnectionRead>(
    mut c: C,
    client: BackupLinkClient,
    state: &S,
) -> Result<Option<C>, ServerError> {
    let Some(message) = c
        .receive_backup_link_message()
        .await
        .change_context(ServerError::Read)?
    else {
        // Client disconnected
        return Ok(None);
    };
    state
        .backup_link_server()
        .receive_message(client, message)
        .await
        .change_context(ServerError::BrokenChannel)?;
    Ok(Some(c))
}

async fn handle_write<C: ClientConnectionWrite>(
    mut c: C,
    receiver: &mut BackupLinkConnectionReceiver,
) -> Result<(), ServerError> {
    loop {
        if let Some(Quit) = handle_write_single(&mut c, receiver).await? {
            return Ok(());
        }
    }
}

async fn handle_write_single<C: ClientConnectionWrite>(
    c: &mut C,
    receiver: &mut BackupLinkConnectionReceiver,
) -> Result<Option<Quit>, ServerError> {
    let Some(message) = receiver.receiver.recv().await else {
        return Ok(Some(Quit));
    };

    c.send_backup_link_message(message)
        .await
        .change_context(ServerError::Write)?;

    Ok(None)
}

struct Quit;
