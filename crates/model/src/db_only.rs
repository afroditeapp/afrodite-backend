use crate::{FcmDeviceToken, PendingNotificationFlags};

#[derive(Debug)]
pub struct PushNotificationStateInfo {
    pub fcm_device_token: Option<FcmDeviceToken>,
    pub fcm_data_notification_sent: bool,
    pub fcm_visible_notification_sent: bool,
}

pub enum PushNotificationStateInfoWithFlags {
    EmptyFlags,
    WithFlags {
        info: PushNotificationStateInfo,
        flags: PendingNotificationFlags,
    },
}
