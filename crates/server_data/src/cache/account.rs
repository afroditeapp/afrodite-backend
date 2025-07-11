use model_server_data::{NewsIteratorSessionIdInternal, PublicationId};

use crate::cache::db_iterator::new_count::DbIteratorNewCount;

#[derive(Debug, Default)]
pub struct CacheAccount {
    pub news_iterator: DbIteratorNewCount<NewsIteratorSessionIdInternal, PublicationId>,
}
