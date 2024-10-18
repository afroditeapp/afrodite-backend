# ContentProcessingState

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cid** | Option<[**models::ContentId**](ContentId.md)> | Content ID of the processed content. | [optional]
**state** | [**models::ContentProcessingStateType**](ContentProcessingStateType.md) |  | 
**wait_queue_position** | Option<**i64**> | Current position in processing queue.  If ProcessingContentId is added to empty queue, then this will be 1. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


