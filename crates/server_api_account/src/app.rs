
use model_account::AccessibleAccount;
use model_server_state::{
    DemoModeConfirmLoginResult, DemoModeId, DemoModeLoginResult, DemoModeLoginToken, DemoModePassword, DemoModeToken
};
use server_api::{DataError, S};
use server_data_account::demo::{AccessibleAccountsInfoUtils, DemoModeUtils};

pub use server_api::app::*;

pub trait DemoModeManagerProvider: StateBase {
    fn stage0_login(
        &self,
        password: DemoModePassword,
    ) -> impl std::future::Future<Output = error_stack::Result<DemoModeLoginResult, DataError>> + Send;

    fn stage1_login(
        &self,
        password: DemoModePassword,
        token: DemoModeLoginToken,
    ) -> impl std::future::Future<Output = error_stack::Result<DemoModeConfirmLoginResult, DataError>>
           + Send;

    fn demo_mode_token_exists(
        &self,
        token: &DemoModeToken,
    ) -> impl std::future::Future<Output = error_stack::Result<DemoModeId, DataError>> + Send;

    fn demo_mode_logout(
        &self,
        token: &DemoModeToken,
    ) -> impl std::future::Future<Output = error_stack::Result<(), DataError>> + Send;

    fn accessible_accounts_if_token_valid(
        &self,
        token: &DemoModeToken,
    ) -> impl std::future::Future<
        Output = server_common::result::Result<Vec<AccessibleAccount>, DataError>,
    > + Send;
}

impl DemoModeManagerProvider for S {
    async fn stage0_login(
        &self,
        password: DemoModePassword,
    ) -> error_stack::Result<DemoModeLoginResult, DataError> {
        self.demo_mode().stage0_login(password).await
    }

    async fn stage1_login(
        &self,
        password: DemoModePassword,
        token: DemoModeLoginToken,
    ) -> error_stack::Result<DemoModeConfirmLoginResult, DataError> {
        self.demo_mode().stage1_login(password, token).await
    }

    async fn demo_mode_token_exists(
        &self,
        token: &DemoModeToken,
    ) -> error_stack::Result<DemoModeId, DataError> {
        self.demo_mode().demo_mode_token_exists(token).await
    }

    async fn demo_mode_logout(
        &self,
        token: &DemoModeToken,
    ) -> error_stack::Result<(), DataError> {
        self.demo_mode().demo_mode_logout(token).await
    }

    async fn accessible_accounts_if_token_valid(
        &self,
        token: &DemoModeToken,
    ) -> server_common::result::Result<Vec<AccessibleAccount>, DataError> {
        let info = self
            .demo_mode()
            .accessible_accounts_if_token_valid(token)
            .await?;
        let accounts = info.into_accounts(self.read()).await?;
        DemoModeUtils::with_extra_info(accounts, self.config(), self.read()).await
    }
}
