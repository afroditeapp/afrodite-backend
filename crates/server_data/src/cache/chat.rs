use limit::ChatLimits;
use model_server_data::{MatchId, MatchesIteratorSessionIdInternal, ReceivedLikeId, ReceivedLikesIteratorSessionIdInternal};

use super::db_iterator::{new_count::DbIteratorNewCount, DbIterator};

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
