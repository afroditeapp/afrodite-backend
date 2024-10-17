use limit::ChatLimits;
use model::{MatchId, MatchesIteratorSessionIdInternal, NextNumberStorage};
use received_likes::ReceivedLikesIterator;

use super::db_iterator::{DbIterator, IteratorSessionIdTrait, IteratorStartPoint};

pub mod limit;
pub mod received_likes;

#[derive(Debug, Default)]
pub struct CachedChatComponentData {
    pub limits: ChatLimits,
    // This cached version of FcmDeviceToken is now disabled
    // as some extra mapping other way aroud would be needed as
    // same FcmDeviceToken might be used for different account if
    // user logs out and logs in with different account.
    // pub fcm_device_token: Option<FcmDeviceToken>,
    pub received_likes_iterator: ReceivedLikesIterator,
    pub matches_iterator: DbIterator<MatchesIteratorSessionIdInternal, MatchId>,
}

impl IteratorSessionIdTrait for MatchesIteratorSessionIdInternal {
    fn create(storage: &mut NextNumberStorage) -> Self {
        MatchesIteratorSessionIdInternal::create(storage)
    }
}

impl IteratorStartPoint for MatchId {}
