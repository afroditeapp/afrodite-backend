use model::{NewsId, NewsIteratorSessionIdInternal, NextNumberStorage};

use super::db_iterator::{DbIterator, IteratorSessionIdTrait, IteratorStartPoint};


#[derive(Debug, Default)]
pub struct CachedAccountComponentData {
    pub news_iterator: DbIterator<NewsIteratorSessionIdInternal, NewsId>,
}

impl IteratorSessionIdTrait for NewsIteratorSessionIdInternal {
    fn create(storage: &mut NextNumberStorage) -> Self {
        NewsIteratorSessionIdInternal::create(storage)
    }
}

impl IteratorStartPoint for NewsId {}
