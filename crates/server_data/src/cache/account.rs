use model_account::{NewsIteratorSessionIdInternal, PublicationId};

use crate::cache::db_iterator::new_count::DbIteratorNewCount;


#[derive(Debug, Default)]
pub struct CachedAccountComponentData {
    pub news_iterator: DbIteratorNewCount<NewsIteratorSessionIdInternal, PublicationId>,
}
