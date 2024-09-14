# \MediaAdminApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**patch_moderation_request_list**](MediaAdminApi.md#patch_moderation_request_list) | **PATCH** /6GF9AybnmCb3J1d4ZfTT95UoiSg | Get current list of moderation requests in my moderation queue.
[**post_handle_moderation_request**](MediaAdminApi.md#post_handle_moderation_request) | **POST** /SiEktmT-jyNLA69x7qffV8c0YUk/{aid} | Handle moderation request of some account.



## patch_moderation_request_list

> models::ModerationList patch_moderation_request_list(queue)
Get current list of moderation requests in my moderation queue.

Additional requests will be added to my queue if necessary.  ## Access  Account with `admin_moderate_images` capability is required to access this route. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**queue** | [**ModerationQueueType**](.md) |  | [required] |

### Return type

[**models::ModerationList**](ModerationList.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_handle_moderation_request

> post_handle_moderation_request(aid, handle_moderation_request)
Handle moderation request of some account.

## Access  Account with `admin_moderate_images` capability is required to access this route. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**aid** | **uuid::Uuid** |  | [required] |
**handle_moderation_request** | [**HandleModerationRequest**](HandleModerationRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

