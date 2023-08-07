# \MediainternalApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**internal_get_check_moderation_request_for_account**](MediainternalApi.md#internal_get_check_moderation_request_for_account) | **GET** /internal/media_api/moderation/request/{account_id} | Check that current moderation request for account exists. Requires also
[**internal_post_update_profile_image_visibility**](MediainternalApi.md#internal_post_update_profile_image_visibility) | **POST** /internal/media_api/visiblity/{account_id}/{value} | 



## internal_get_check_moderation_request_for_account

> internal_get_check_moderation_request_for_account(account_id)
Check that current moderation request for account exists. Requires also

Check that current moderation request for account exists. Requires also that request contains camera image. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## internal_post_update_profile_image_visibility

> internal_post_update_profile_image_visibility(account_id, value, profile)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |
**value** | **bool** |  | [required] |
**profile** | [**Profile**](Profile.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

