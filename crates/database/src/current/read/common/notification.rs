use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountIdInternal, PendingAppNotification, PendingAppNotificationDb, PendingAppNotificationType,
};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentReadCommonNotification);

impl CurrentReadCommonNotification<'_> {
    pub fn pending_app_notification_type_numbers(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingAppNotificationType>, DieselDatabaseError> {
        use crate::schema::pending_app_notifications::dsl::*;

        pending_app_notifications
            .filter(account_id.eq(id.as_db_id()))
            .select(notification_type_number)
            .order_by(notification_type_number.asc())
            .load(self.conn())
            .into_db_error(id)
    }

    pub fn pending_app_notification_type_numbers_without_sent_push(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingAppNotificationType>, DieselDatabaseError> {
        use crate::schema::pending_app_notifications::dsl::*;

        pending_app_notifications
            .filter(account_id.eq(id.as_db_id()))
            .filter(push_notification_sent.eq(false))
            .select(notification_type_number)
            .order_by(notification_type_number.asc())
            .load(self.conn())
            .into_db_error(id)
    }

    pub fn pending_app_notifications_without_sent_push(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingAppNotification>, DieselDatabaseError> {
        use crate::schema::pending_app_notifications::dsl::*;

        let rows: Vec<PendingAppNotification> = pending_app_notifications
            .filter(account_id.eq(id.as_db_id()))
            .filter(push_notification_sent.eq(false))
            .select(PendingAppNotification::as_select())
            .order_by(notification_type_number.asc())
            .load(self.conn())
            .into_db_error(id)?;

        Ok(rows)
    }

    pub fn pending_app_notifications(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingAppNotification>, DieselDatabaseError> {
        use crate::schema::pending_app_notifications::dsl::*;

        let rows: Vec<PendingAppNotification> = pending_app_notifications
            .filter(account_id.eq(id.as_db_id()))
            .select(PendingAppNotification::as_select())
            .order_by(notification_type_number.asc())
            .load(self.conn())
            .into_db_error(id)?;

        Ok(rows)
    }

    pub fn received_likes_notification_with_unsent_email(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<PendingAppNotificationDb>, DieselDatabaseError> {
        use crate::schema::pending_app_notifications::dsl::*;

        let row = pending_app_notifications
            .filter(account_id.eq(id.as_db_id()))
            .filter(notification_type_number.eq(PendingAppNotificationType::ReceivedLikesChanged))
            .filter(email_notification_sent.eq(false))
            .select(PendingAppNotificationDb::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(id)?;

        Ok(row)
    }
}
