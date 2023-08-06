use std::net::SocketAddr;

use crate::{
    api::model::{AccountIdInternal, AuthPair},
    server::data::DatabaseError,
    utils::ConvertCommandError,
};

use error_stack::Result;

define_write_commands!(WriteCommandsCommon);

impl WriteCommandsCommon<'_> {
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

    /// Remove current connection address, access and refresh tokens.
    pub async fn logout(&self, id: AccountIdInternal) -> Result<(), DatabaseError> {
        self.current()
            .account()
            .update_refresh_token(id, None)
            .await
            .convert(id)?;

        self.end_connection_session(id, true).await?;

        Ok(())
    }

    /// Remove current connection address and access token.
    pub async fn end_connection_session(
        &self,
        id: AccountIdInternal,
        remove_access_token: bool,
    ) -> Result<(), DatabaseError> {
        let current_access_token = if remove_access_token {
            self.current_write()
                .read()
                .account()
                .access_token(id)
                .await
                .convert(id)?
        } else {
            None
        };

        self.cache()
            .delete_access_token_and_connection(id.as_light(), current_access_token)
            .await
            .convert(id)?;

        self.current()
            .account()
            .update_api_key(id, None)
            .await
            .convert(id)?;

        Ok(())
    }
}
