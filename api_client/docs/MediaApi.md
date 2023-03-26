# \MediaApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_image**](MediaApi.md#get_image) | **GET** /media_api/image/{account_id}/{image_file} | Get profile image
[**get_moderation_request**](MediaApi.md#get_moderation_request) | **GET** /media_api/moderation/request | Get current moderation request.
[**get_moderation_request_list**](MediaApi.md#get_moderation_request_list) | **GET** /media_api/admin/moderation/page/next | Get list of next moderation requests in moderation queue.
[**post_handle_moderation_request**](MediaApi.md#post_handle_moderation_request) | **POST** /media_api/admin/moderation/handle_request/{request_id} | Handle moderation request.
[**put_image_to_moderation_slot**](MediaApi.md#put_image_to_moderation_slot) | **PUT** /media_api/moderation/request/slot/{slot_id} | Set image to moderation request slot.
[**put_moderation_request**](MediaApi.md#put_moderation_request) | **PUT** /media_api/moderation/request | Create new or override old moderation request.



## get_image

> get_image(account_id, image_file)
Get profile image

Get profile image

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |
**image_file** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_moderation_request

> crate::models::ModerationRequest get_moderation_request()
Get current moderation request.

Get current moderation request. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ModerationRequest**](ModerationRequest.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_moderation_request_list

> crate::models::ModerationRequestList get_moderation_request_list()
Get list of next moderation requests in moderation queue.

Get list of next moderation requests in moderation queue.  ## Access  Account with `admin_moderate_images` capability is required to access this route. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ModerationRequestList**](ModerationRequestList.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_handle_moderation_request

> post_handle_moderation_request(request_id, handle_moderation_request)
Handle moderation request.

Handle moderation request.  ## Access  Account with `admin_moderate_images` capability is required to access this route. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_id** | **uuid::Uuid** |  | [required] |
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

> put_image_to_moderation_slot(slot_id, body)
Set image to moderation request slot.

Set image to moderation request slot.  Slots \"camera\" and \"image1\" are available. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**slot_id** | **String** |  | [required] |
**body** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: image/jpeg
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_moderation_request

> put_moderation_request(new_moderation_request)
Create new or override old moderation request.

Create new or override old moderation request.  Set images to moderation request slots first. 

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

