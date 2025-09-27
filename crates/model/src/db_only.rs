use crate::{FcmDeviceToken, PendingNotificationFlags, PushNotificationDbState};

#[derive(Debug)]
pub struct PushNotificationStateInfo {
    pub fcm_device_token: Option<FcmDeviceToken>,
}

pub enum PushNotificationStateInfoWithFlags {
    EmptyFlags,
    WithFlags {
        info: PushNotificationStateInfo,
        flags: PendingNotificationFlags,
    },
}

pub struct PushNotificationSendingInfo {
    pub db_state: PushNotificationDbState,
    pub flags: PendingNotificationFlags,
}
