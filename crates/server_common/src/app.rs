use model::{AccountId, AccountIdInternal, ServerVersion};
use simple_backend_utils::Result;

use crate::data::DataError;

pub trait ServerVersionProvider {
    fn server_version(&self) -> ServerVersion;
}

/// All accounts registered in the service.
pub trait GetAccounts {
    fn get_internal_id(
        &self,
        id: AccountId,
    ) -> impl std::future::Future<Output = Result<AccountIdInternal, DataError>> + Send;

    fn get_internal_id_optional(
        &self,
        id: AccountId,
    ) -> impl std::future::Future<Output = Option<AccountIdInternal>> + Send;
}
