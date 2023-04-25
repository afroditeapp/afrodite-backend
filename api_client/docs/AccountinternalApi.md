# \AccountinternalApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_api_key**](AccountinternalApi.md#check_api_key) | **GET** /internal/check_api_key | 
[**internal_get_account_state**](AccountinternalApi.md#internal_get_account_state) | **GET** /internal/get_account_state/{account_id} | 



## check_api_key

> crate::models::AccountIdLight check_api_key(api_key)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**api_key** | [**ApiKey**](ApiKey.md) |  | [required] |

### Return type

[**crate::models::AccountIdLight**](AccountIdLight.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## internal_get_account_state

> crate::models::Account internal_get_account_state(account_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |

### Return type

[**crate::models::Account**](Account.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

