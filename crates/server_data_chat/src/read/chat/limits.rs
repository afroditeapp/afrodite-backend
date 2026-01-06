use database_chat::current::read::GetDbReadCommandsChat;
use error_stack::ResultExt;
use model::{AccountIdInternal, UnixTime};
use model_chat::DailyLikesLeftInternal;
use server_data::{
    DataError,
    db_manager::InternalReading,
    define_cmd_wrapper_read,
    read::DbRead,
    result::{Result, WrappedContextExt},
};
use simple_backend_utils::time::next_possible_utc_date_time_value;

define_cmd_wrapper_read!(ReadCommandsChatLimits);

pub struct ResetDailyLikes {
    pub new_value: i16,
}

impl ReadCommandsChatLimits<'_> {
    pub async fn is_daily_likes_left_reset_needed(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<ResetDailyLikes>, DataError> {
        let Some(config) = self
            .config()
            .client_features_internal()
            .likes
            .daily
            .as_ref()
        else {
            return Ok(None);
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
            Ok(Some(ResetDailyLikes {
                new_value: config.daily_likes.into(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn daily_likes_left_internal(
        &self,
        account: AccountIdInternal,
    ) -> Result<DailyLikesLeftInternal, DataError> {
        let data = self
            .db_read(move |mut cmds| cmds.chat().limits().daily_likes_left(account))
            .await?;
        Ok(data)
    }
}
