use crate::models::AccountIdLight;

impl Copy for AccountIdLight {}

impl AccountIdLight {
    pub fn to_string(&self) -> String {
        self.account_id.hyphenated().to_string()
    }
}
