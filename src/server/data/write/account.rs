use std::net::SocketAddr;

use crate::{
    api::model::{AccountIdInternal, AuthPair},
    server::data::DatabaseError,
    utils::ConvertCommandError,
};

use error_stack::Result;

define_write_commands!(WriteCommandsAccount);

impl WriteCommandsAccount<'_> {
    pub async fn set_new_auth_pair(
        &self,
        id: AccountIdInternal,
        pair: AuthPair,
        address: Option<SocketAddr>,
    ) -> Result<(), DatabaseError> {
        let current_access_token = self
            .current_write()
            .read()
            .account()
            .access_token(id)
            .await
            .convert(id)?;

        self.current()
            .account()
            .update_api_key(id, Some(&pair.access))
            .await
            .convert(id)?;

        self.current()
            .account()
            .update_refresh_token(id, Some(&pair.refresh))
            .await
            .convert(id)?;

        self.cache()
            .update_access_token_and_connection(
                id.as_light(),
                current_access_token,
                pair.access,
                address,
            )
            .await
            .convert(id)
    }
}
