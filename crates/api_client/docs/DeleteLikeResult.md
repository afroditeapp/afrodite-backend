# DeleteLikeResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**error_account_interaction_state_mismatch** | Option<[**models::CurrentAccountInteractionState**](CurrentAccountInteractionState.md)> |  | [optional]
**error_delete_already_done_before** | Option<**bool**> | The account tracking for delete like only tracks the latest deleter account, so it is possible that this error resets if delete like target account likes and removes the like. | [optional][default to false]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


