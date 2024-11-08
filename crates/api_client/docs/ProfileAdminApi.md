# \ProfileAdminApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_profile_name_pending_moderation_list**](ProfileAdminApi.md#get_profile_name_pending_moderation_list) | **GET** /82woXm_Kq9yEtRHP7KAcXkgRWnU | 
[**get_profile_statistics_history**](ProfileAdminApi.md#get_profile_statistics_history) | **GET** /6CGbSNdoURdJRTBxb3Hb_OGw9ME | 
[**get_profile_text_pending_moderation_list**](ProfileAdminApi.md#get_profile_text_pending_moderation_list) | **GET** /pdEU3ussEDsELfe6TOtjqrDojOc | Get first page of pending profile text moderations. Oldest item is first and count 25.
[**post_moderate_profile_name**](ProfileAdminApi.md#post_moderate_profile_name) | **POST** /bnrAbC2DpwIftQouXUAVR1W6g8Y | 
[**post_moderate_profile_text**](ProfileAdminApi.md#post_moderate_profile_text) | **POST** /53BBFzgF9dZhb7_HvZSqLidsqbg | 



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


## get_profile_text_pending_moderation_list

> models::GetProfileTextPendingModerationList get_profile_text_pending_moderation_list(show_texts_which_bots_can_moderate)
Get first page of pending profile text moderations. Oldest item is first and count 25.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**show_texts_which_bots_can_moderate** | **bool** |  | [required] |

### Return type

[**models::GetProfileTextPendingModerationList**](GetProfileTextPendingModerationList.md)

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


## post_moderate_profile_text

> post_moderate_profile_text(post_moderate_profile_text)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_moderate_profile_text** | [**PostModerateProfileText**](PostModerateProfileText.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

