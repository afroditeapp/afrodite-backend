use model::{AccountStateRelatedSharedState, OtherSharedState, PendingNotificationFlags, Permissions};
use model_server_data::AppNotificationSettingsInternal;

use crate::event::EventSender;

use super::ConnectionInfo;

#[derive(Debug)]
pub struct CacheEntryCommon {
    pub permissions: Permissions,
    pub account_state_related_shared_state: AccountStateRelatedSharedState,
    pub other_shared_state: OtherSharedState,
    pub current_connection: Option<ConnectionInfo>,
    /// The cached pending notification flags indicates not yet handled
    /// notification which PushNotificationManager will handle as soon as
    /// possible.
    pub pending_notification_flags: PendingNotificationFlags,
    pub app_notification_settings: AppNotificationSettingsInternal,
}

impl CacheEntryCommon {
    pub fn connection_event_sender(&self) -> Option<&EventSender> {
        self.current_connection
            .as_ref()
            .map(|info| &info.event_sender)
    }
}

impl Default for CacheEntryCommon {
    fn default() -> Self {
        CacheEntryCommon {
            permissions: Permissions::default(),
            account_state_related_shared_state: AccountStateRelatedSharedState::default(),
            other_shared_state: OtherSharedState::default(),
            current_connection: None,
            pending_notification_flags: PendingNotificationFlags::empty(),
            app_notification_settings: AppNotificationSettingsInternal::default(),
        }
    }
}
