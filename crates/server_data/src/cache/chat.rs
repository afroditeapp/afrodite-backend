use std::time::Instant;

use model::AccountIdInternal;

#[derive(Debug, Default)]
pub struct CacheChat {
    // This cached version of PushNotificationDeviceToken is now disabled
    // as some extra mapping other way aroud would be needed as
    // same PushNotificationDeviceToken might be used for different account if
    // user logs out and logs in with different account.
    // pub push_notification_device_token: Option<PushNotificationDeviceToken>,
    pub currently_typing_to: CurrentlyTypingTo,
    pub check_online_status: CheckOnlineStatus,
}

pub enum CurrentlyTypingToAccess<'a> {
    Allowed(&'a mut Option<AccountIdInternal>),
    Denied,
}

#[derive(Debug, Default)]
pub struct CurrentlyTypingTo {
    typing_to: Option<AccountIdInternal>,
    previously_received: Option<Instant>,
}

impl CurrentlyTypingTo {
    pub fn access_typing_to_state(&mut self, min_wait_seconds: u16) -> CurrentlyTypingToAccess {
        let time_elapsed = if let Some(timestamp) = self.previously_received {
            timestamp.elapsed().as_secs() >= min_wait_seconds as u64
        } else {
            true
        };
        if time_elapsed {
            self.previously_received = Some(Instant::now());
            CurrentlyTypingToAccess::Allowed(&mut self.typing_to)
        } else {
            CurrentlyTypingToAccess::Denied
        }
    }
}

#[derive(Debug, Default)]
pub struct CheckOnlineStatus {
    previously_received: Option<Instant>,
}

impl CheckOnlineStatus {
    pub fn check_if_allowed(&mut self, min_wait_seconds: u16) -> bool {
        let time_elapsed = if let Some(timestamp) = self.previously_received {
            timestamp.elapsed().as_secs() >= min_wait_seconds as u64
        } else {
            true
        };
        if time_elapsed {
            self.previously_received = Some(Instant::now());
            true
        } else {
            false
        }
    }
}
