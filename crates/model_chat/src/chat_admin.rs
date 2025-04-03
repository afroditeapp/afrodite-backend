use model::AccountId;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct SetMaxPublicKeyCount {
    pub account: AccountId,
    pub count: i64,
}
