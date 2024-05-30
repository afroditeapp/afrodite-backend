
use model::{AccessibleAccount, DemoModeConfirmLoginResult, DemoModeId, DemoModeLoginResult, DemoModeLoginToken, DemoModePassword, DemoModeToken};
pub use server_api::app::*;
use server_api::DataError;

use server_data::result::Result;

pub trait DemoModeManagerProvider {
    async fn stage0_login(
        &self,
        password: DemoModePassword,
    ) -> Result<DemoModeLoginResult, DataError>;

    async fn stage1_login(
        &self,
        password: DemoModePassword,
        token: DemoModeLoginToken,
    ) -> Result<DemoModeConfirmLoginResult, DataError>;

    async fn demo_mode_token_exists(
        &self,
        token: &DemoModeToken,
    ) -> Result<DemoModeId, DataError>;

    async fn accessible_accounts_if_token_valid<S: GetConfig + GetAccounts + ReadData>(
        &self,
        state: &S,
        token: &DemoModeToken,
    ) -> Result<Vec<AccessibleAccount>, DataError>;
}
