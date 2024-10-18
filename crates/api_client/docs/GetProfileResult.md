# GetProfileResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**lst** | Option<**i64**> | Account's most recent disconnect time.  If the last seen time is not None, then it is Unix timestamp or -1 if the profile is currently online. | [optional]
**p** | Option<[**models::Profile**](Profile.md)> | Profile data if it is newer than the version in the query. | [optional]
**v** | Option<[**models::ProfileVersion**](ProfileVersion.md)> | If empty then profile does not exist or current account does not have access to the profile. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


