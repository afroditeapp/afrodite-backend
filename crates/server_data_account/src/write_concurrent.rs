use model::AccountIdInternal;
use model_account::{NewsIteratorSessionId, PublicationId};
use server_data::{DataError, cache::db_iterator::new_count::DbIteratorStateNewCount};

pub trait WriteConcurrentAccount {
    fn next_news_iterator_state(
        &self,
        id: AccountIdInternal,
        iterator_session_id: NewsIteratorSessionId,
    ) -> Result<Option<DbIteratorStateNewCount<PublicationId>>, DataError>;
}
