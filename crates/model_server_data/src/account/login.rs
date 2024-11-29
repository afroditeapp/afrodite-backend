use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_string_wrapper;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SignInWithInfo {
    pub google_account_id: Option<GoogleAccountId>,
}

impl SignInWithInfo {
    pub fn google_account_id_matches_with(&self, id: &GoogleAccountId) -> bool {
        if let Some(google_account_id) = &self.google_account_id {
            google_account_id == id
        } else {
            false
        }
    }

    pub fn some_sign_in_with_method_is_set(&self) -> bool {
        self.google_account_id.is_some()
    }
}

#[derive(
    Debug, Serialize, Deserialize, Clone, PartialEq, diesel::FromSqlRow, diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
#[serde(transparent)]
pub struct GoogleAccountId(pub String);

impl GoogleAccountId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

diesel_string_wrapper!(GoogleAccountId);
