use database::current::read::GetDbReadCommandsCommon;
use database_account::current::read::GetDbReadCommandsAccount;
use model::UnixTime;
use model_account::{
    AccountGlobalState, AccountId, AccountIdInternal, AccountSetup, AppleAccountId, BotAccount,
    EmailAddress, EmailAddressState, EmailAddressStateInternal, EmailLoginTokens, GetBotsResult,
    GoogleAccountId, SignInWithInfo,
};
use model_server_state::DemoAccountId;
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

pub mod ban;
pub mod delete;
pub mod email;
pub mod news;
pub mod notification;

define_cmd_wrapper_read!(ReadCommandsAccount);

impl<'a> ReadCommandsAccount<'a> {
    pub fn ban(self) -> ban::ReadCommandsAccountBan<'a> {
        ban::ReadCommandsAccountBan::new(self.0)
    }

    pub fn delete(self) -> delete::ReadCommandsAccountDelete<'a> {
        delete::ReadCommandsAccountDelete::new(self.0)
    }

    pub fn email(self) -> email::ReadCommandsAccountEmail<'a> {
        email::ReadCommandsAccountEmail::new(self.0)
    }

    pub fn news(self) -> news::ReadCommandsAccountNews<'a> {
        news::ReadCommandsAccountNews::new(self.0)
    }

    pub fn notification(self) -> notification::ReadCommandsAccountNotification<'a> {
        notification::ReadCommandsAccountNotification::new(self.0)
    }
}

impl ReadCommandsAccount<'_> {
    pub async fn account_sign_in_with_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<SignInWithInfo, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account()
                .sign_in_with()
                .sign_in_with_info_raw(id)
                .map(|v| v.into())
        })
        .await
        .into_error()
    }

    pub async fn is_bot_account(&self, id: AccountIdInternal) -> Result<bool, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common()
                .state()
                .other_shared_state(id)
                .map(|v| v.is_bot_account)
        })
        .await
        .into_error()
    }

    pub async fn email_address_state(
        &self,
        id: AccountIdInternal,
    ) -> Result<EmailAddressState, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().email_address_state(id))
            .await
            .into_error()
    }

    pub async fn email_address_state_internal(
        &self,
        id: AccountIdInternal,
    ) -> Result<EmailAddressStateInternal, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().email_address_state_internal(id))
            .await
            .into_error()
    }

    pub async fn email_verification_token(
        &self,
        id: AccountIdInternal,
    ) -> Result<(Option<Vec<u8>>, Option<UnixTime>), DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_verification_token(id))
            .await
            .into_error()
    }

    pub async fn email_verification_token_time(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<UnixTime>, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_verification_token_time(id))
            .await
            .into_error()
    }

    pub async fn email_login_tokens(
        &self,
        id: AccountIdInternal,
    ) -> Result<EmailLoginTokens, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_login_tokens(id))
            .await
            .into_error()
    }

    pub async fn email_login_token_time(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<UnixTime>, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_login_token_time(id))
            .await
            .into_error()
    }

    pub async fn account_setup(&self, id: AccountIdInternal) -> Result<AccountSetup, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().account_setup(id))
            .await
            .into_error()
    }

    pub async fn apple_account_id_to_account_id(
        &self,
        id: AppleAccountId,
    ) -> Result<Option<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account()
                .sign_in_with()
                .apple_account_id_to_account_id(id)
        })
        .await
        .into_error()
    }

    pub async fn google_account_id_to_account_id(
        &self,
        id: GoogleAccountId,
    ) -> Result<Option<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account()
                .sign_in_with()
                .google_account_id_to_account_id(id)
        })
        .await
        .into_error()
    }

    pub async fn demo_account_owned_account_ids(
        &self,
        id: DemoAccountId,
    ) -> Result<Vec<AccountId>, DataError> {
        self.db_read(move |mut cmds| cmds.account().demo().owned_account_ids(id))
            .await
            .into_error()
    }

    pub async fn global_state(&self) -> Result<AccountGlobalState, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().global_state())
            .await
            .into_error()
    }

    /// Get existing bot accounts by email pattern.
    /// Admin bot is admin@example.com, user bots are botNUMBER@example.com.
    pub async fn get_existing_bots(&self) -> Result<GetBotsResult, DataError> {
        const ADMIN_EMAIL: &str = "admin@example.com";
        const BOT_EMAIL_PREFIX: &str = "bot";
        const BOT_EMAIL_SUFFIX: &str = "@example.com";

        // Get bot config to determine expected user bot count
        let bot_config = self
            .db_read(move |mut cmds| {
                cmds.common()
                    .bot_config()
                    .bot_config()
                    .map(|v| v.unwrap_or_default())
            })
            .await?;
        let expected_user_count = bot_config.user_bots as usize;

        // Query admin bot
        let admin = if bot_config.admin_bot {
            self.get_existing_bot_account(EmailAddress(ADMIN_EMAIL.to_string()))
                .await?
        } else {
            None
        };

        // Query user bots
        let mut users = Vec::new();
        for i in 0..expected_user_count {
            let bot_email = EmailAddress(format!("{}{}{}", BOT_EMAIL_PREFIX, i, BOT_EMAIL_SUFFIX));
            if let Some(bot) = self.get_existing_bot_account(bot_email).await? {
                users.push(bot);
            }
        }

        Ok(GetBotsResult { admin, users })
    }

    /// Helper to get a single existing bot account by email
    async fn get_existing_bot_account(
        &self,
        email: EmailAddress,
    ) -> Result<Option<BotAccount>, DataError> {
        let email_clone = email.clone();
        let result: Option<AccountIdInternal> = self
            .db_read(move |mut cmds| cmds.account().email().account_id_from_email(email_clone))
            .await?;

        match result {
            Some(internal_id) => {
                let aid: AccountId = internal_id.as_id();
                Ok(Some(BotAccount { aid }))
            }
            None => Ok(None),
        }
    }
}
