# \MediaApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_image**](MediaApi.md#get_image) | **GET** /media_api/image/{account_id}/{content_id} | Get profile image
[**get_moderation_request**](MediaApi.md#get_moderation_request) | **GET** /media_api/moderation/request | Get current moderation request.
[**patch_moderation_request_list**](MediaApi.md#patch_moderation_request_list) | **PATCH** /media_api/admin/moderation/page/next | Get current list of moderation requests in my moderation queue.
[**post_handle_moderation_request**](MediaApi.md#post_handle_moderation_request) | **POST** /media_api/admin/moderation/handle_request/{account_id} | Handle moderation request of some account.
[**put_image_to_moderation_slot**](MediaApi.md#put_image_to_moderation_slot) | **PUT** /media_api/moderation/request/slot/{slot_id} | Set image to moderation request slot.
[**put_moderation_request**](MediaApi.md#put_moderation_request) | **PUT** /media_api/moderation/request | Create new or override old moderation request.



## get_image

> std::path::PathBuf get_image(account_id, content_id)
Get profile image

Get profile image

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |
**content_id** | **uuid::Uuid** |  | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: image/jpeg

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_moderation_request

> crate::models::NewModerationRequest get_moderation_request()
Get current moderation request.

Get current moderation request. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::NewModerationRequest**](NewModerationRequest.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## patch_moderation_request_list

> crate::models::ModerationList patch_moderation_request_list()
Get current list of moderation requests in my moderation queue.

Get current list of moderation requests in my moderation queue. Additional requests will be added to my queue if necessary.  ## Access  Account with `admin_moderate_images` capability is required to access this route. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ModerationList**](ModerationList.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_handle_moderation_request

> post_handle_moderation_request(account_id, handle_moderation_request)
Handle moderation request of some account.

Handle moderation request of some account.  ## Access  Account with `admin_moderate_images` capability is required to access this route. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |
**handle_moderation_request** | [**HandleModerationRequest**](HandleModerationRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_image_to_moderation_slot

> crate::models::ContentId put_image_to_moderation_slot(slot_id, body)
Set image to moderation request slot.

Set image to moderation request slot.  Slots from 0 to 2 are available.  TODO: resize and check images at some point 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**slot_id** | **i32** |  | [required] |
**body** | **std::path::PathBuf** |  | [required] |

### Return type

[**crate::models::ContentId**](ContentId.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: image/jpeg
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_moderation_request

> put_moderation_request(new_moderation_request)
Create new or override old moderation request.

Create new or override old moderation request.  Make sure that moderation request has content IDs which points to your own image slots. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**new_moderation_request** | [**NewModerationRequest**](NewModerationRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

