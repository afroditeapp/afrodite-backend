# \AccountInternalApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_access_token**](AccountInternalApi.md#check_access_token) | **GET** /internal/check_access_token | 
[**internal_get_account_state**](AccountInternalApi.md#internal_get_account_state) | **GET** /internal/get_account_state/{account_id} | 



## check_access_token

> crate::models::AccountId check_access_token(access_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**access_token** | [**AccessToken**](AccessToken.md) |  | [required] |

### Return type

[**crate::models::AccountId**](AccountId.md)

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

