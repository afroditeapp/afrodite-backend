# SendMessageResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**error_receiver_public_key_outdated** | Option<**bool**> |  | [optional][default to false]
**error_sender_message_id_was_not_expected_id** | Option<[**models::SenderMessageId**](SenderMessageId.md)> |  | [optional]
**error_too_many_pending_messages** | Option<**bool**> |  | [optional][default to false]
**message_number** | Option<**i64**> | None if error happened | [optional]
**unix_time** | Option<**i64**> | None if error happened | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


