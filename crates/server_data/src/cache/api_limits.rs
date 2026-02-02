use core::fmt;

#[derive(Default)]
pub struct ApiLimitState {
    value: u16,
}

impl fmt::Debug for ApiLimitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiLimitState").finish()
    }
}

impl ApiLimitState {
    pub fn increment_and_check_is_limit_reached(&mut self, limit: u16) -> bool {
        if self.value < limit {
            self.value += 1;
            false
        } else {
            true
        }
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn value(&self) -> u16 {
        self.value
    }
}

#[derive(Debug, Default)]
pub struct AllApiLimits {
    // Common
    pub send_report: ApiLimitState,
    // Profile
    pub post_reset_profile_paging: ApiLimitState,
    pub post_get_next_profile_page: ApiLimitState,
    pub get_profile: ApiLimitState,
    // Media
    pub get_profile_content_info: ApiLimitState,
    // Chat
    pub post_send_message: ApiLimitState,
}
