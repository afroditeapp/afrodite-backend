# \ProfileAdminApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_profile_name_pending_moderation_list**](ProfileAdminApi.md#get_profile_name_pending_moderation_list) | **GET** /82woXm_Kq9yEtRHP7KAcXkgRWnU | 
[**get_profile_statistics_history**](ProfileAdminApi.md#get_profile_statistics_history) | **GET** /6CGbSNdoURdJRTBxb3Hb_OGw9ME | 
[**post_moderate_profile_name**](ProfileAdminApi.md#post_moderate_profile_name) | **POST** /bnrAbC2DpwIftQouXUAVR1W6g8Y | 



## get_profile_name_pending_moderation_list

> models::GetProfileNamePendingModerationList get_profile_name_pending_moderation_list()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetProfileNamePendingModerationList**](GetProfileNamePendingModerationList.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_profile_statistics_history

> models::GetProfileStatisticsHistoryResult get_profile_statistics_history(value_type, age)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**value_type** | [**ProfileStatisticsHistoryValueType**](.md) |  | [required] |
**age** | Option<**i64**> | Required only for AgeChange history |  |

### Return type

[**models::GetProfileStatisticsHistoryResult**](GetProfileStatisticsHistoryResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_moderate_profile_name

> post_moderate_profile_name(post_moderate_profile_name)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_moderate_profile_name** | [**PostModerateProfileName**](PostModerateProfileName.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

