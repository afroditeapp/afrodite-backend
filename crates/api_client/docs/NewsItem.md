# NewsItem

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**aid_creator** | Option<[**models::AccountId**](AccountId.md)> | Only visible for accounts which have some news permissions | [optional]
**aid_editor** | Option<[**models::AccountId**](AccountId.md)> | Only visible for accounts which have some news permissions | [optional]
**body** | **String** |  | 
**creation_time** | [**models::UnixTime**](UnixTime.md) |  | 
**edit_time** | Option<[**models::UnixTime**](UnixTime.md)> |  | [optional]
**locale** | **String** |  | 
**title** | **String** |  | 
**version** | Option<[**models::NewsTranslationVersion**](NewsTranslationVersion.md)> | Only visible for accounts which have some news permissions | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


