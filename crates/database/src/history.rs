use model::AccountIdLight;
use time::OffsetDateTime;

pub mod read;
pub mod write;

pub struct HistoryData<T> {
    row_id: i64,
    account_id: AccountIdLight,
    unix_time: OffsetDateTime,
    data: T,
}
