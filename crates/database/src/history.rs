use model::AccountId;
use time::OffsetDateTime;

pub mod read;
pub mod write;

pub struct HistoryData<T> {
    row_id: i64,
    account_id: AccountId,
    unix_time: OffsetDateTime,
    data: T,
}
