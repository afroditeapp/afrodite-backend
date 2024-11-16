# NewsItem

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**aid_creator** | Option<[**models::AccountId**](AccountId.md)> | Only visible for accounts which have some news permissions | [optional]
**aid_editor** | Option<[**models::AccountId**](AccountId.md)> | Only visible for accounts which have some news permissions | [optional]
**body** | **String** |  | 
**edit_unix_time** | Option<**i64**> | Option<i64> is a workaround for Dart OpenApi generator version 7.9.0 | [optional]
**locale** | **String** |  | 
**time** | Option<[**models::UnixTime**](UnixTime.md)> | Latest publication time | [optional]
**title** | **String** |  | 
**version** | Option<[**models::NewsTranslationVersion**](NewsTranslationVersion.md)> | Only visible for accounts which have some news permissions | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


