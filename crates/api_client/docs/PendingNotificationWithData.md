# PendingNotificationWithData

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**new_message_received_from** | Option<[**Vec<crate::models::AccountId>**](AccountId.md)> | Data for NEW_MESSAGE notification.  List of account IDs which have sent a new message. | [optional]
**value** | **i64** | Pending notification (or multiple notifications which each have different type) not yet received notifications which push notification requests client to download.  The integer is a bitflag.  - const NEW_MESSAGE = 0x1;  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


