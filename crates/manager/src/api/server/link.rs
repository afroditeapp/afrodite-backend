use std::net::SocketAddr;
use manager_api::protocol::ClientConnectionRead;
use manager_api::protocol::ClientConnectionWrite;

use manager_model::ManagerInstanceName;
use tracing::info;
use crate::api::utils::validate_json_rpc_link_login;
use crate::api::GetJsonRcpLinkManager;

use manager_api::protocol::{ConnectionUtilsRead, ConnectionUtilsWrite};

use crate::server::link::json_rpc::server::JsonRpcLinkConnectionReceiver;

use crate::server::app::S;

use error_stack::{Result, ResultExt};
use super::ClientConnectionReadWrite;
use super::ServerError;

pub async fn handle_json_rpc_link<
    C: ClientConnectionReadWrite,
>(
    mut c: C,
    address: SocketAddr,
    state: S,
) -> Result<(), ServerError> {
    let name = c.receive_string_with_u32_len()
        .await
        .change_context(ServerError::Read)?;
    let name = ManagerInstanceName(name);

    let password = c.receive_string_with_u32_len()
        .await
        .change_context(ServerError::Read)?;

    if validate_json_rpc_link_login(&state, address, &name, &password).is_err() {
        c.send_u8(0).await.change_context(ServerError::Write)?;
        return Ok(());
    }

    c.send_u8(1).await.change_context(ServerError::Write)?;

    handle_server_messages(c, address, state, name).await
}

async fn handle_server_messages<
    C: ClientConnectionReadWrite,
>(
    c: C,
    address: SocketAddr,
    state: S,
    name: ManagerInstanceName,
) -> Result<(), ServerError> {
    info!("Remote manager {} from {} started JSON RPC link connection", name, address);

    let (reader, writer) = tokio::io::split(c);

    let mut receiver = state.json_rpc_link_server().replace_connection()
        .await
        .change_context(ServerError::JsonRpcLink)?;

    let r = tokio::select! {
        r = handle_read(reader, state.clone()) => {
            r
        }
        r = handle_write(writer, &mut receiver) => {
            r
        },
    };

    drop(receiver);
    state.json_rpc_link_server().clean_connection()
        .await
        .change_context(ServerError::JsonRpcLink)?;

    info!("Remote manager {} from {} disconnected JSON RPC link connection", name, address);

    r
}

async fn handle_read<
    C: ClientConnectionRead,
>(
    mut c: C,
    state: S,
) -> Result<(), ServerError> {
    loop {
        let Some(connection) = handle_read_one_message(
            c,
            &state,
        ).await? else {
            return Ok(());
        };
        c = connection;
    }
}

async fn handle_read_one_message<
    C: ClientConnectionRead,
>(
    mut c: C,
    state: &S,
) -> Result<Option<C>, ServerError> {
    let Some(message) = c.receive_json_rpc_link_message().await.change_context(ServerError::Read)? else {
        // Client disconnected
        return Ok(None);
    };
    state.json_rpc_link_server().receive_message(message).await.change_context(ServerError::BrokenChannel)?;
    Ok(Some(c))
}

async fn handle_write<
    C: ClientConnectionWrite,
>(
    mut c: C,
    receiver: &mut JsonRpcLinkConnectionReceiver,
) -> Result<(), ServerError> {
    loop {
        if let Some(Quit) = handle_write_single(&mut c, receiver).await? {
            return Ok(());
        }
    }
}

async fn handle_write_single<
    C: ClientConnectionWrite,
>(
    c: &mut C,
    receiver: &mut JsonRpcLinkConnectionReceiver,
) -> Result<Option<Quit>, ServerError> {
    let Some(message) = receiver.receiver.recv()
        .await else {
            return Ok(Some(Quit));
        };

    c.send_json_rpc_link_message(message).await.change_context(ServerError::Write)?;

    Ok(None)
}

struct Quit;
