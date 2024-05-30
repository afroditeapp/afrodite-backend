use crate::FcmDeviceToken;

#[derive(Debug)]
pub struct PushNotificationStateInfo {
    pub fcm_device_token: Option<FcmDeviceToken>,
    pub fcm_notification_sent: bool,
}
