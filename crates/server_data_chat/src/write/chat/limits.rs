use database_chat::current::{read::GetDbReadCommandsChat, write::GetDbWriteCommandsChat};
use model::{AccountIdInternal, UnixTime};
use server_data::{
    DataError,
    app::GetConfig,
    define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt, WrappedResultExt},
    write::DbTransaction,
};
use simple_backend_utils::time::next_possible_utc_date_time_value;

define_cmd_wrapper_write!(WriteCommandsChatLimits);

impl WriteCommandsChatLimits<'_> {
    pub async fn reset_daily_likes_left_if_needed(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        let Some(config) = self
            .config()
            .client_features()
            .and_then(|v| v.limits.likes.like_sending.as_ref())
        else {
            return Ok(());
        };

        let limit = self
            .db_read(move |mut cmds| cmds.chat().limits().daily_likes_left(id))
            .await?;
        let reset = if let Some(latest_reset) = limit.latest_limit_reset_unix_time {
            // Avoid reseting the time again after reset done by
            // DailyLikesManager because most likely after that
            // the latest_reset matches config.reset_time and
            // next_possible_utc_date_time_value will return the latest_reset
            // in that case.
            let latest_reset = latest_reset.add_seconds(1);
            let latest_reset = latest_reset
                .to_chrono_time()
                .ok_or(DataError::Time.report())?;
            let next_reset: UnixTime =
                next_possible_utc_date_time_value(latest_reset, config.reset_time)
                    .change_context(DataError::Time)?
                    .into();
            let current_time = UnixTime::current_time();
            current_time.ut >= next_reset.ut
        } else {
            true
        };

        if reset {
            let limit_value = config.daily_limit;
            db_transaction!(self, move |mut cmds| {
                cmds.chat()
                    .limits()
                    .reset_daily_likes_left(id, limit_value.into())
            })?;
        }

        Ok(())
    }

    pub async fn reset_daily_likes_left(
        &self,
        id: AccountIdInternal,
        value: i64,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().limits().reset_daily_likes_left(id, value)
        })?;
        Ok(())
    }

    pub async fn decrement_daily_likes_left(&self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().limits().decrement_daily_likes_left(id)
        })?;
        Ok(())
    }

    pub async fn reset_daily_likes_left_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().limits().reset_daily_likes_left_sync_version(id)
        })?;
        Ok(())
    }
}
