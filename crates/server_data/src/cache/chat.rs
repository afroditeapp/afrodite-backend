#[derive(Debug, Default)]
pub struct CacheChat {
    // This cached version of FcmDeviceToken is now disabled
    // as some extra mapping other way aroud would be needed as
    // same FcmDeviceToken might be used for different account if
    // user logs out and logs in with different account.
    // pub fcm_device_token: Option<FcmDeviceToken>,
}
