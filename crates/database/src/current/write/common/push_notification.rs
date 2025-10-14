use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    AccountIdInternal, PushNotificationDeviceToken, PushNotificationEncryptionKey,
    PushNotificationFlagsDb, SyncVersion, UnixTime,
};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentWriteCommonPushNotification);

impl CurrentWriteCommonPushNotification<'_> {
    pub fn remove_push_notification_device_token_and_encryption_key(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::push_notification::dsl::*;

        update(push_notification.find(id.as_db_id()))
            .set((
                device_token.eq(None::<PushNotificationDeviceToken>),
                device_token_unix_time.eq(None::<UnixTime>),
                encryption_key.eq(None::<PushNotificationEncryptionKey>),
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
        use model::schema::push_notification::dsl::*;

        update(push_notification.find(id.as_db_id()))
            .set((
                device_token.eq(None::<PushNotificationDeviceToken>),
                device_token_unix_time.eq(None::<UnixTime>),
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
    ) -> Result<PushNotificationEncryptionKey, DieselDatabaseError> {
        use model::schema::push_notification::dsl::*;

        // Remove the token from other accounts. It is possible that
        // same device is used for multiple accounts.
        update(push_notification.filter(device_token.eq(token.clone())))
            .set((
                device_token.eq(None::<PushNotificationDeviceToken>),
                device_token_unix_time.eq(None::<UnixTime>),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        let notification_token = PushNotificationEncryptionKey::generate_new();

        update(push_notification.find(id.as_db_id()))
            .set((
                device_token.eq(token),
                device_token_unix_time.eq(UnixTime::current_time()),
                encryption_key.eq(notification_token.clone()),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        self.increment_push_notification_info_sync_version(id)?;

        Ok(notification_token)
    }

    pub fn save_current_notification_flags_to_database_if_needed(
        &mut self,
        id: AccountIdInternal,
        current_flags: PushNotificationFlagsDb,
        current_sent_flags: PushNotificationFlagsDb,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::push_notification::dsl::*;

        let (db_pending_flags, db_sent_flags): (i64, i64) = push_notification
            .filter(account_id.eq(id.as_db_id()))
            .select((pending_flags, sent_flags))
            .first(self.conn())
            .into_db_error(())?;

        if db_pending_flags != *current_flags.as_i64() {
            update(push_notification.find(id.as_db_id()))
                .set(pending_flags.eq(current_flags))
                .execute(self.conn())
                .into_db_error(())?;
        }

        if db_sent_flags != *current_sent_flags.as_i64() {
            update(push_notification.find(id.as_db_id()))
                .set(sent_flags.eq(current_sent_flags))
                .execute(self.conn())
                .into_db_error(())?;
        }

        Ok(())
    }

    pub fn reset_push_notification_info_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::push_notification::dsl::*;

        update(push_notification)
            .filter(account_id.eq(id.as_db_id()))
            .set(sync_version.eq(0))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn increment_push_notification_info_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::push_notification::dsl::*;

        update(push_notification)
            .filter(account_id.eq(id.as_db_id()))
            .filter(sync_version.lt(SyncVersion::MAX_VALUE))
            .set(sync_version.eq(sync_version + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn increment_push_notification_info_sync_version_for_every_account(
        &mut self,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::push_notification::dsl::*;

        update(push_notification)
            .filter(sync_version.lt(SyncVersion::MAX_VALUE))
            .set(sync_version.eq(sync_version + 1))
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
