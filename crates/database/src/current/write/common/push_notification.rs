use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    AccountIdInternal, PendingNotification, PendingNotificationToken, PushNotificationDeviceToken,
    SyncVersion, UnixTime,
};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentWriteCommonPushNotification);

impl CurrentWriteCommonPushNotification<'_> {
    pub fn remove_push_notification_device_token_and_pending_notification_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state.find(id.as_db_id()))
            .set((
                push_notification_device_token.eq(None::<PushNotificationDeviceToken>),
                push_notification_device_token_unix_time.eq(None::<UnixTime>),
                pending_notification_token.eq(None::<PendingNotificationToken>),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        self.increment_push_notification_info_sync_version(id)?;

        Ok(())
    }

    pub fn remove_push_notification_device_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state.find(id.as_db_id()))
            .set((
                push_notification_device_token.eq(None::<PushNotificationDeviceToken>),
                push_notification_device_token_unix_time.eq(None::<UnixTime>),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        self.increment_push_notification_info_sync_version(id)?;

        Ok(())
    }

    pub fn update_push_notification_device_token_and_generate_new_notification_token(
        &mut self,
        id: AccountIdInternal,
        token: PushNotificationDeviceToken,
    ) -> Result<PendingNotificationToken, DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        // Remove the token from other accounts. It is possible that
        // same device is used for multiple accounts.
        update(common_state.filter(push_notification_device_token.eq(token.clone())))
            .set((
                push_notification_device_token.eq(None::<PushNotificationDeviceToken>),
                push_notification_device_token_unix_time.eq(None::<UnixTime>),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        let notification_token = PendingNotificationToken::generate_new();

        update(common_state.find(id.as_db_id()))
            .set((
                push_notification_device_token.eq(token),
                push_notification_device_token_unix_time.eq(UnixTime::current_time()),
                pending_notification_token.eq(notification_token.clone()),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        self.increment_push_notification_info_sync_version(id)?;

        Ok(notification_token)
    }

    pub fn get_and_reset_pending_notification_with_notification_token(
        &mut self,
        token: PendingNotificationToken,
    ) -> Result<(AccountIdInternal, PendingNotification), DieselDatabaseError> {
        use model::schema::{account_id, common_state};

        let token_clone = token.clone();
        let (id, notification) = common_state::table
            .inner_join(account_id::table)
            .filter(common_state::pending_notification_token.eq(token_clone))
            .select((
                AccountIdInternal::as_select(),
                common_state::pending_notification,
            ))
            .first(self.conn())
            .into_db_error(())?;

        update(common_state::table.filter(common_state::pending_notification_token.eq(token)))
            .set((common_state::pending_notification.eq(0),))
            .execute(self.conn())
            .into_db_error(())?;

        Ok((id, notification))
    }

    pub fn save_current_notification_flags_to_database_if_needed(
        &mut self,
        id: AccountIdInternal,
        current_flags: PendingNotification,
        current_sent_flags: PendingNotification,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        let (flags, sent_flags): (i64, i64) = common_state
            .filter(account_id.eq(id.as_db_id()))
            .select((pending_notification, pending_notification_sent))
            .first(self.conn())
            .into_db_error(())?;

        if flags != *current_flags.as_i64() {
            update(common_state.find(id.as_db_id()))
                .set(pending_notification.eq(current_flags))
                .execute(self.conn())
                .into_db_error(())?;
        }

        if sent_flags != *current_sent_flags.as_i64() {
            update(common_state.find(id.as_db_id()))
                .set(pending_notification_sent.eq(current_sent_flags))
                .execute(self.conn())
                .into_db_error(())?;
        }

        Ok(())
    }

    pub fn reset_push_notification_info_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(push_notification_info_sync_version.eq(0))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn increment_push_notification_info_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state)
            .filter(account_id.eq(id.as_db_id()))
            .filter(push_notification_info_sync_version.lt(SyncVersion::MAX_VALUE))
            .set(push_notification_info_sync_version.eq(push_notification_info_sync_version + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn increment_push_notification_info_sync_version_for_every_account(
        &mut self,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state)
            .filter(push_notification_info_sync_version.lt(SyncVersion::MAX_VALUE))
            .set(push_notification_info_sync_version.eq(push_notification_info_sync_version + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn upsert_vapid_public_key_hash(
        &mut self,
        sha256_vapid_public_key_hash: &str,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::vapid_public_key_hash::dsl::*;

        insert_into(vapid_public_key_hash)
            .values((row_type.eq(0), sha256_hash.eq(sha256_vapid_public_key_hash)))
            .on_conflict(row_type)
            .do_update()
            .set(sha256_hash.eq(sha256_vapid_public_key_hash))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
