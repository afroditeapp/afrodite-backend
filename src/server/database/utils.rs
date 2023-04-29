

use error_stack::Result;




use crate::api::model::{
    Account, AccountIdInternal, AccountIdLight, AccountSetup, ApiKey, Profile, ProfileUpdateInternal, ProfileInternal,
};

use super::{
    cache::{CacheError, DatabaseCache},
    read::ReadCmd,
    write::{WriteCmd},
};

// pub trait GetReadWriteCmd {
//     fn read_cmd(id: AccountIdInternal) -> ReadCmd;
//     fn write_cmd(id: AccountIdInternal) -> WriteCmd;
// }

// impl GetReadWriteCmd for Account {
//     fn read_cmd(id: AccountIdInternal) -> ReadCmd {
//         ReadCmd::AccountState(id)
//     }

//     fn write_cmd(id: AccountIdInternal) -> WriteCmd {
//         WriteCmd::AccountState(id)
//     }
// }

// impl GetReadWriteCmd for AccountSetup {
//     fn read_cmd(id: AccountIdInternal) -> ReadCmd {
//         ReadCmd::AccountSetup(id)
//     }

//     fn write_cmd(id: AccountIdInternal) -> WriteCmd {
//         WriteCmd::AccountSetup(id)
//     }
// }

// impl GetReadWriteCmd for ProfileUpdateInternal {
//     fn read_cmd(id: AccountIdInternal) -> ReadCmd {
//         ReadCmd::Profile(id)
//     }

//     fn write_cmd(id: AccountIdInternal) -> WriteCmd {
//         WriteCmd::ProfileUpdate(id)
//     }
// }

// impl GetReadWriteCmd for ProfileInternal {
//     fn read_cmd(id: AccountIdInternal) -> ReadCmd {
//         ReadCmd::Profile(id)
//     }

//     fn write_cmd(id: AccountIdInternal) -> WriteCmd {
//         WriteCmd::ProfileUpdate(id)
//     }
// }

pub fn current_unix_time() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

pub struct ApiKeyManager<'a> {
    cache: &'a DatabaseCache,
}

impl<'a> ApiKeyManager<'a> {
    pub fn new(cache: &'a DatabaseCache) -> Self {
        Self { cache }
    }

    pub async fn api_key_exists(&self, api_key: &ApiKey) -> Option<AccountIdInternal> {
        self.cache.api_key_exists(api_key).await
    }

    // pub async fn update_api_key(&self, id: AccountIdLight, api_key: ApiKey) -> Result<(), CacheError> {
    //     self.cache.update_api_key(id, api_key).await
    // }

    // pub async fn delete_api_key(&self, api_key: ApiKey) -> Result<(), CacheError> {
    //     self.cache.delete_api_key(api_key).await
    // }
}

pub struct AccountIdManager<'a> {
    cache: &'a DatabaseCache,
}

impl<'a> AccountIdManager<'a> {
    pub fn new(cache: &'a DatabaseCache) -> Self {
        Self { cache }
    }

    pub async fn get_internal_id(
        &self,
        id: AccountIdLight,
    ) -> Result<AccountIdInternal, CacheError> {
        self.cache.to_account_id_internal(id).await
    }
}
