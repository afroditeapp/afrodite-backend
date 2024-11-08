# PendingNotificationWithData

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content_moderation_request_completed** | Option<[**models::ModerationRequestState**](ModerationRequestState.md)> | Data for CONTENT_MODERATION_REQUEST_COMPLETED notification. | [optional]
**new_message_received_from** | Option<[**Vec<models::AccountId>**](AccountId.md)> | Data for NEW_MESSAGE notification.  List of account IDs which have sent a new message. | [optional]
**news_changed** | Option<[**models::UnreadNewsCountResult**](UnreadNewsCountResult.md)> | Data for NEWS_CHANGED notification. | [optional]
**received_likes_changed** | Option<[**models::NewReceivedLikesCountResult**](NewReceivedLikesCountResult.md)> | Data for RECEIVED_LIKES_CHANGED notification. | [optional]
**value** | **i64** | Pending notification (or multiple notifications which each have different type) not yet received notifications which push notification requests client to download.  The integer is a bitflag.  - const NEW_MESSAGE = 0x1; - const RECEIVED_LIKES_CHANGED = 0x2; - const CONTENT_MODERATION_REQUEST_COMPLETED = 0x4; - const NEWS_CHANGED = 0x8;  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


