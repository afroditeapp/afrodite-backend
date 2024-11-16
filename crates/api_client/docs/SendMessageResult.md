# SendMessageResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**error_receiver_blocked_sender_or_receiver_not_found** | Option<**bool**> |  | [optional][default to false]
**error_receiver_public_key_outdated** | Option<**bool**> |  | [optional][default to false]
**error_too_many_receiver_acknowledgements_missing** | Option<**bool**> |  | [optional][default to false]
**error_too_many_sender_acknowledgements_missing** | Option<**bool**> |  | [optional][default to false]
**mn** | Option<[**models::MessageNumber**](MessageNumber.md)> | None if error happened | [optional]
**ut** | Option<[**models::UnixTime**](UnixTime.md)> | None if error happened | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


