use model::{
    AccessToken, AccessTokenUnixTime, AccountStateRelatedSharedState, IpAddressInternal,
    LoginSession, OtherSharedState, Permissions, PushNotificationFlags, RefreshToken,
};
use model_server_data::{AppNotificationSettingsInternal, AuthPair};

use super::ConnectionInfo;
use crate::{cache::api_limits::AllApiLimits, event::EventSender};

#[derive(Debug)]
pub struct CacheCommon {
    pub permissions: Permissions,
    pub account_state_related_shared_state: AccountStateRelatedSharedState,
    pub other_shared_state: OtherSharedState,
    pub current_connection: Option<ConnectionInfo>,
    login_session: Option<LoginSession>,
    login_session_changed: bool,
    /// The cached pending push notification flags indicates not yet handled
    /// notification which PushNotificationManager will handle as soon as
    /// possible.
    pub pending_push_notification_flags: PushNotificationFlags,
    pub sent_push_notification_flags: PushNotificationFlags,
    pub app_notification_settings: AppNotificationSettingsInternal,
    api_limits: AllApiLimits,
}

impl CacheCommon {
    pub fn load_from_db(&mut self, data: Option<LoginSession>) {
        self.login_session = data;
    }

    pub fn update_tokens(
        &mut self,
        auth_pair: AuthPair,
        access_token_ip_address: IpAddressInternal,
    ) {
        self.login_session = Some(LoginSession {
            access_token: auth_pair.access,
            access_token_unix_time: AccessTokenUnixTime::current_time(),
            access_token_ip_address,
            refresh_token: auth_pair.refresh,
        });
        self.login_session_changed = true;
    }

    pub fn logout(&mut self) {
        self.login_session = None;
        self.login_session_changed = true;
    }

    pub fn get_tokens_if_save_needed(&mut self) -> Option<Option<LoginSession>> {
        if self.login_session_changed {
            Some(self.login_session.clone())
        } else {
            None
        }
    }

    pub fn access_token(&self) -> Option<&AccessToken> {
        self.login_session.as_ref().map(|v| &v.access_token)
    }

    pub fn access_token_unix_time(&self) -> Option<AccessTokenUnixTime> {
        self.login_session
            .as_ref()
            .map(|v| v.access_token_unix_time)
    }

    pub fn access_token_ip_address(&self) -> Option<IpAddressInternal> {
        self.login_session
            .as_ref()
            .map(|v| v.access_token_ip_address)
    }

    pub fn refresh_token(&self) -> Option<&RefreshToken> {
        self.login_session.as_ref().map(|v| &v.refresh_token)
    }

    pub fn connection_event_sender(&self) -> Option<&EventSender> {
        self.current_connection
            .as_ref()
            .map(|info| &info.event_sender)
    }

    pub fn api_limits(&mut self) -> &mut AllApiLimits {
        &mut self.api_limits
    }

    pub fn reset_api_limits(&mut self) {
        self.api_limits = AllApiLimits::default();
    }
}

impl Default for CacheCommon {
    fn default() -> Self {
        CacheCommon {
            permissions: Permissions::default(),
            account_state_related_shared_state: AccountStateRelatedSharedState::default(),
            other_shared_state: OtherSharedState::default(),
            current_connection: None,
            login_session: None,
            login_session_changed: false,
            pending_push_notification_flags: PushNotificationFlags::empty(),
            sent_push_notification_flags: PushNotificationFlags::empty(),
            app_notification_settings: AppNotificationSettingsInternal::default(),
            api_limits: AllApiLimits::default(),
        }
    }
}
