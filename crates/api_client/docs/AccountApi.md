# \AccountApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_cancel_deletion**](AccountApi.md#delete_cancel_deletion) | **DELETE** /account_api/delete | Cancel account deletion.
[**get_account_data**](AccountApi.md#get_account_data) | **GET** /account_api/account_data | Get changeable user information to account.
[**get_account_setup**](AccountApi.md#get_account_setup) | **GET** /account_api/account_setup | Get non-changeable user information to account.
[**get_account_state**](AccountApi.md#get_account_state) | **GET** /account_api/state | Get current account state.
[**get_deletion_status**](AccountApi.md#get_deletion_status) | **GET** /account_api/delete | Get deletion status.
[**post_account_data**](AccountApi.md#post_account_data) | **POST** /account_api/account_data | Set changeable user information to account.
[**post_account_setup**](AccountApi.md#post_account_setup) | **POST** /account_api/account_setup | Setup non-changeable user information during `initial setup` state.
[**post_complete_setup**](AccountApi.md#post_complete_setup) | **POST** /account_api/complete_setup | Complete initial setup.
[**post_delete**](AccountApi.md#post_delete) | **PUT** /account_api/delete | Delete account.
[**post_login**](AccountApi.md#post_login) | **POST** /account_api/login | Get new AccessToken.
[**post_register**](AccountApi.md#post_register) | **POST** /account_api/register | Register new account. Returns new account ID which is UUID.
[**post_sign_in_with_login**](AccountApi.md#post_sign_in_with_login) | **POST** /account_api/sign_in_with_login | Start new session with sign in with Apple or Google. Creates new account if
[**put_setting_profile_visiblity**](AccountApi.md#put_setting_profile_visiblity) | **PUT** /account_api/settings/profile_visibility | Update current or pending profile visiblity value.



## delete_cancel_deletion

> delete_cancel_deletion()
Cancel account deletion.

Cancel account deletion.  Account state will move to previous state.

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_account_data

> crate::models::AccountData get_account_data()
Get changeable user information to account.

Get changeable user information to account.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::AccountData**](AccountData.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_account_setup

> crate::models::AccountSetup get_account_setup()
Get non-changeable user information to account.

Get non-changeable user information to account.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::AccountSetup**](AccountSetup.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_account_state

> crate::models::Account get_account_state()
Get current account state.

Get current account state.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::Account**](Account.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_deletion_status

> crate::models::DeleteStatus get_deletion_status()
Get deletion status.

Get deletion status.  Get information when account will be really deleted.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::DeleteStatus**](DeleteStatus.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_account_data

> post_account_data(account_data)
Set changeable user information to account.

Set changeable user information to account.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_data** | [**AccountData**](AccountData.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_account_setup

> post_account_setup(account_setup)
Setup non-changeable user information during `initial setup` state.

Setup non-changeable user information during `initial setup` state.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_setup** | [**AccountSetup**](AccountSetup.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_complete_setup

> post_complete_setup()
Complete initial setup.

Complete initial setup.  Requirements: - Account must be in `InitialSetup` state. - Account must have a moderation request. - The current or pending security image of the account is in the request. - The current or pending first profile image of the account is in the request. 

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_delete

> post_delete()
Delete account.

Delete account.  Changes account state to `pending deletion` from all possible states. Previous state will be saved, so it will be possible to stop automatic deletion process.

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_login

> crate::models::LoginResult post_login(account_id)
Get new AccessToken.

Get new AccessToken.  Available only if server is running in debug mode and bot_login is enabled from config file.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | [**AccountId**](AccountId.md) |  | [required] |

### Return type

[**crate::models::LoginResult**](LoginResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_register

> crate::models::AccountId post_register()
Register new account. Returns new account ID which is UUID.

Register new account. Returns new account ID which is UUID.  Available only if server is running in debug mode and bot_login is enabled from config file.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::AccountId**](AccountId.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_sign_in_with_login

> crate::models::LoginResult post_sign_in_with_login(sign_in_with_login_info)
Start new session with sign in with Apple or Google. Creates new account if

Start new session with sign in with Apple or Google. Creates new account if it does not exists.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sign_in_with_login_info** | [**SignInWithLoginInfo**](SignInWithLoginInfo.md) |  | [required] |

### Return type

[**crate::models::LoginResult**](LoginResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_setting_profile_visiblity

> put_setting_profile_visiblity(boolean_setting)
Update current or pending profile visiblity value.

Update current or pending profile visiblity value.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**boolean_setting** | [**BooleanSetting**](BooleanSetting.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

