use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

pub struct ApiUsage {
    // Common
    pub get_connect_websocket: AtomicU32,
    // Profile
    pub post_get_next_profile_page: AtomicU32,
    pub post_reset_profile_paging: AtomicU32,
    pub post_automatic_profile_search_get_next_profile_page: AtomicU32,
    pub post_automatic_profile_search_reset_profile_paging: AtomicU32,
    pub get_profile: AtomicU32,
    // Media
    pub get_content: AtomicU32,
    pub get_profile_content_info: AtomicU32,
    // Chat
    pub post_send_like: AtomicU32,
    pub post_send_message: AtomicU32,
    pub get_video_call_urls: AtomicU32,
}

impl Default for ApiUsage {
    fn default() -> Self {
        Self {
            get_connect_websocket: AtomicU32::new(0),
            post_get_next_profile_page: AtomicU32::new(0),
            post_reset_profile_paging: AtomicU32::new(0),
            post_automatic_profile_search_get_next_profile_page: AtomicU32::new(0),
            post_automatic_profile_search_reset_profile_paging: AtomicU32::new(0),
            get_profile: AtomicU32::new(0),
            get_content: AtomicU32::new(0),
            get_profile_content_info: AtomicU32::new(0),
            post_send_like: AtomicU32::new(0),
            post_send_message: AtomicU32::new(0),
            get_video_call_urls: AtomicU32::new(0),
        }
    }
}

pub struct ApiUsageValue {
    pub name: &'static str,
    pub value: u32,
}

macro_rules! to_api_usage_values {
    ($s:expr, $($field:ident ,)*) => {
        [
            $(
                ApiUsageValue {
                    name: stringify!($field),
                    value: $s.$field.load(Ordering::Relaxed),
                },
            )*
        ]
    };
}

impl ApiUsage {
    pub fn values(&self) -> impl Iterator<Item=ApiUsageValue> {
        let array = to_api_usage_values!(
            self,
            get_connect_websocket,
            post_get_next_profile_page,
            post_reset_profile_paging,
            post_automatic_profile_search_get_next_profile_page,
            post_automatic_profile_search_reset_profile_paging,
            get_profile,
            get_content,
            get_profile_content_info,
            post_send_like,
            post_send_message,
            get_video_call_urls,
        );

        array.into_iter()
    }
}
