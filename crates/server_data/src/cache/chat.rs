#[derive(Debug, Default)]
pub struct CacheChat {
    // This cached version of PushNotificationDeviceToken is now disabled
    // as some extra mapping other way aroud would be needed as
    // same PushNotificationDeviceToken might be used for different account if
    // user logs out and logs in with different account.
    // pub push_notification_device_token: Option<PushNotificationDeviceToken>,
}
