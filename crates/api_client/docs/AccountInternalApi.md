# \AccountInternalApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**post_login**](AccountInternalApi.md#post_login) | **POST** /account_api/login | Get new AccessToken for a bot account. If the account is not registered
[**post_register**](AccountInternalApi.md#post_register) | **POST** /account_api/register | Register a new bot account. Returns new account ID which is UUID.



## post_login

> models::LoginResult post_login(account_id)
Get new AccessToken for a bot account. If the account is not registered

Get new AccessToken for a bot account. If the account is not registered as a bot account, then the request will fail.  Available only if server internal API is enabled with bot_login from config file.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | [**AccountId**](AccountId.md) |  | [required] |

### Return type

[**models::LoginResult**](LoginResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_register

> models::AccountId post_register()
Register a new bot account. Returns new account ID which is UUID.

Register a new bot account. Returns new account ID which is UUID.  Available only if server internal API is enabled with bot_login from config file.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::AccountId**](AccountId.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

