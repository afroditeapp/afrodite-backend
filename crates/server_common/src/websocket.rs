#[derive(thiserror::Error, Debug)]
pub enum WebSocketError {
    #[error("Receive error")]
    Receive,
    #[error("Client sent something unsupported")]
    ProtocolError,
    #[error("Client version is unsupported")]
    ClientVersionUnsupported,
    #[error("Received wrong refresh token")]
    ReceiveWrongRefreshToken,
    #[error("Websocket data sending error")]
    Send,
    #[error("Websocket closing failed")]
    Close,
    #[error("Data serialization error")]
    Serialize,

    // Database errors
    #[error("Database: No refresh token")]
    DatabaseNoRefreshToken,
    #[error("Invalid refresh token in database")]
    InvalidRefreshTokenInDatabase,
    #[error("Database: account logout failed")]
    DatabaseLogoutFailed,
    #[error("Database: saving new tokens failed or other error")]
    DatabaseSaveTokensOrOtherError,
    #[error("Database: Account state query failed")]
    DatabaseAccountStateQuery,
    #[error("Database: Chat state query failed")]
    DatabaseChatStateQuery,
    #[error("Database: Profile state query failed")]
    DatabaseProfileStateQuery,
    #[error("Database: News count state query failed")]
    DatabaseNewsCountQuery,
    #[error("Database: Media content sync version query failed")]
    DatabaseMediaContentSyncVersionQuery,
    #[error("Database: Daily likes left sync version query failed")]
    DatabaseDailyLikesLeftSyncVersionQuery,
    #[error("Database: Pending messages query failed")]
    DatabasePendingMessagesQuery,
    #[error("Database: Profile text moderation completed notification query failed")]
    DatabaseProfileTextModerationCompletedNotificationQuery,
    #[error("Database: Automatic profile search completed notification query failed")]
    DatabaseAutomaticProfileSearchCompletedNotificationQuery,
    #[error("Database: Media content moderation completed notification query failed")]
    DatabaseMediaContentModerationCompletedNotificationQuery,
    #[error("Database: Reset FCM notification sent booleans failed")]
    DatabaseResetFcmNotificationSentBooleans,
    #[error("Database: Bot and gender info query failed")]
    DatabaseBotAndGenderInfoQuery,

    // Event errors
    #[error("Event channel creation failed")]
    EventChannelCreationFailed,

    // Sync
    #[error("Account data version number reset failed")]
    AccountDataVersionResetFailed,
    #[error("Chat data version number reset failed")]
    ChatDataVersionResetFailed,
    #[error("Profile attributes sync version number reset failed")]
    ProfileAttributesSyncVersionResetFailed,
    #[error("Profile sync version number reset failed")]
    ProfileSyncVersionResetFailed,
    #[error("News count sync version number reset failed")]
    NewsCountSyncVersionResetFailed,
    #[error("Media content sync version number reset failed")]
    MediaContentSyncVersionResetFailed,
    #[error("Daily likes left sync version number reset failed")]
    DailyLikesLeftSyncVersionResetFailed,
}
