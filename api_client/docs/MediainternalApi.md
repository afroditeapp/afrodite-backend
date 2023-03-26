# \MediainternalApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**post_image**](MediainternalApi.md#post_image) | **POST** /internal/image/{account_id}/{image_file} | 



## post_image

> post_image(account_id, image_file, image_file2)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |
**image_file** | **String** |  | [required] |
**image_file2** | [**ImageFile**](ImageFile.md) | Upload new image | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: image/jpeg
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

