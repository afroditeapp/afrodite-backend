use limit::ChatLimits;
use model::{MatchId, MatchesIteratorSessionIdInternal, NextNumberStorage, ReceivedLikeId, ReceivedLikesIteratorSessionIdInternal};

use super::db_iterator::{new_count::DbIteratorNewCount, DbIterator, IteratorSessionIdTrait, IteratorStartPoint};

pub mod limit;

#[derive(Debug, Default)]
pub struct CachedChatComponentData {
    pub limits: ChatLimits,
    // This cached version of FcmDeviceToken is now disabled
    // as some extra mapping other way aroud would be needed as
    // same FcmDeviceToken might be used for different account if
    // user logs out and logs in with different account.
    // pub fcm_device_token: Option<FcmDeviceToken>,
    pub received_likes_iterator: DbIteratorNewCount<ReceivedLikesIteratorSessionIdInternal, ReceivedLikeId>,
    pub matches_iterator: DbIterator<MatchesIteratorSessionIdInternal, MatchId>,
}

impl IteratorSessionIdTrait for ReceivedLikesIteratorSessionIdInternal {
    fn create(storage: &mut NextNumberStorage) -> Self {
        ReceivedLikesIteratorSessionIdInternal::create(storage)
    }
}

impl IteratorStartPoint for ReceivedLikeId {}

impl IteratorSessionIdTrait for MatchesIteratorSessionIdInternal {
    fn create(storage: &mut NextNumberStorage) -> Self {
        MatchesIteratorSessionIdInternal::create(storage)
    }
}

impl IteratorStartPoint for MatchId {}
