use model::NextNumberStorage;
use model_account::{NewsIteratorSessionIdInternal, PublicationId};

use super::db_iterator::{new_count::DbIteratorNewCount, IteratorSessionIdTrait, IteratorStartPoint};


#[derive(Debug, Default)]
pub struct CachedAccountComponentData {
    pub news_iterator: DbIteratorNewCount<NewsIteratorSessionIdInternal, PublicationId>,
}

impl IteratorSessionIdTrait for NewsIteratorSessionIdInternal {
    fn create(storage: &mut NextNumberStorage) -> Self {
        NewsIteratorSessionIdInternal::create(storage)
    }
}

impl IteratorStartPoint for PublicationId {}
