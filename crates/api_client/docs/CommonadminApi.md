# \CommonadminApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_latest_build_info**](CommonadminApi.md#get_latest_build_info) | **GET** /common_api/get_latest_build_info | Get latest software build information available for update from manager
[**get_software_info**](CommonadminApi.md#get_software_info) | **GET** /common_api/software_info | Get software version information from manager instance.
[**get_system_info**](CommonadminApi.md#get_system_info) | **GET** /common_api/system_info | Get system information from manager instance.
[**post_request_build_software**](CommonadminApi.md#post_request_build_software) | **POST** /common_api/request_build_software | Request building new software from manager instance.
[**post_request_update_software**](CommonadminApi.md#post_request_update_software) | **POST** /common_api/request_update_software | Request updating new software from manager instance.



## get_latest_build_info

> crate::models::BuildInfo get_latest_build_info(software_options)
Get latest software build information available for update from manager

Get latest software build information available for update from manager instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**software_options** | [**SoftwareOptions**](.md) |  | [required] |

### Return type

[**crate::models::BuildInfo**](BuildInfo.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_software_info

> crate::models::SoftwareInfo get_software_info()
Get software version information from manager instance.

Get software version information from manager instance.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::SoftwareInfo**](SoftwareInfo.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_system_info

> crate::models::SystemInfoList get_system_info()
Get system information from manager instance.

Get system information from manager instance.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::SystemInfoList**](SystemInfoList.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_request_build_software

> post_request_build_software(software_options)
Request building new software from manager instance.

Request building new software from manager instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**software_options** | [**SoftwareOptions**](.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_request_update_software

> post_request_update_software(software_options, reboot)
Request updating new software from manager instance.

Request updating new software from manager instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**software_options** | [**SoftwareOptions**](.md) |  | [required] |
**reboot** | **bool** |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

