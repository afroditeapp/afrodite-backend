use model::{
    AccessToken, AccountStateRelatedSharedState, OtherSharedState, PendingNotificationFlags,
    Permissions, RefreshToken,
};
use model_server_data::{AppNotificationSettingsInternal, AuthPair};

use super::ConnectionInfo;
use crate::event::EventSender;

#[derive(Debug)]
pub struct CacheCommon {
    pub permissions: Permissions,
    pub account_state_related_shared_state: AccountStateRelatedSharedState,
    pub other_shared_state: OtherSharedState,
    pub current_connection: Option<ConnectionInfo>,
    access_token: Option<AccessToken>,
    refresh_token: Option<RefreshToken>,
    tokens_changed: bool,
    /// The cached pending notification flags indicates not yet handled
    /// notification which PushNotificationManager will handle as soon as
    /// possible.
    pub pending_notification_flags: PendingNotificationFlags,
    pub app_notification_settings: AppNotificationSettingsInternal,
}

impl CacheCommon {
    pub fn load_from_db(
        &mut self,
        access_token: Option<AccessToken>,
        refresh_token: Option<RefreshToken>,
    ) {
        self.access_token = access_token;
        self.refresh_token = refresh_token;
    }

    pub fn update_tokens(&mut self, auth_pair: AuthPair) {
        self.access_token = Some(auth_pair.access);
        self.refresh_token = Some(auth_pair.refresh);
        self.tokens_changed = true;
    }

    pub fn logout(&mut self) {
        self.access_token = None;
        self.refresh_token = None;
        self.tokens_changed = true;
    }

    pub fn get_tokens_if_save_needed(
        &mut self,
    ) -> Option<(Option<AccessToken>, Option<RefreshToken>)> {
        if self.tokens_changed {
            None
        } else {
            Some((self.access_token.clone(), self.refresh_token.clone()))
        }
    }

    pub fn access_token(&self) -> Option<&AccessToken> {
        self.access_token.as_ref()
    }

    pub fn refresh_token(&self) -> Option<&RefreshToken> {
        self.refresh_token.as_ref()
    }

    pub fn connection_event_sender(&self) -> Option<&EventSender> {
        self.current_connection
            .as_ref()
            .map(|info| &info.event_sender)
    }
}

impl Default for CacheCommon {
    fn default() -> Self {
        CacheCommon {
            permissions: Permissions::default(),
            account_state_related_shared_state: AccountStateRelatedSharedState::default(),
            other_shared_state: OtherSharedState::default(),
            current_connection: None,
            access_token: None,
            refresh_token: None,
            tokens_changed: false,
            pending_notification_flags: PendingNotificationFlags::empty(),
            app_notification_settings: AppNotificationSettingsInternal::default(),
        }
    }
}
