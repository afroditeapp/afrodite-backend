# \AccountApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_cancel_deletion**](AccountApi.md#delete_cancel_deletion) | **DELETE** /account_api/delete | Cancel account deletion.
[**get_account_state**](AccountApi.md#get_account_state) | **GET** /account_api/state | Get current account state.
[**get_deletion_status**](AccountApi.md#get_deletion_status) | **GET** /account_api/delete | Get deletion status.
[**post_account_setup**](AccountApi.md#post_account_setup) | **POST** /account_api/setup | Setup non-changeable user information during `initial setup` state.
[**post_complete_setup**](AccountApi.md#post_complete_setup) | **POST** /account_api/complete_setup | Complete initial setup.
[**post_delete**](AccountApi.md#post_delete) | **PUT** /account_api/delete | Delete account.
[**post_login**](AccountApi.md#post_login) | **POST** /account_api/login | Get new ApiKey.
[**post_register**](AccountApi.md#post_register) | **POST** /account_api/register | Register new account. Returns new account ID which is UUID.
[**post_sign_in_with_login**](AccountApi.md#post_sign_in_with_login) | **POST** /account_api/sign_in_with_login | Start new session with sign in with Apple or Google. Creates new account if
[**put_setting_profile_visiblity**](AccountApi.md#put_setting_profile_visiblity) | **PUT** /account_api/settings/profile_visibility | Update profile visiblity value.



## delete_cancel_deletion

> delete_cancel_deletion()
Cancel account deletion.

Cancel account deletion.  Account state will move to previous state.

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

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

[api_key](../README.md#api_key)

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

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

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

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_complete_setup

> post_complete_setup()
Complete initial setup.

Complete initial setup.  Request to this handler will complete if client is in `initial setup`, setup information is set and image moderation request has been made. 

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

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

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_login

> crate::models::LoginResult post_login(account_id_light)
Get new ApiKey.

Get new ApiKey.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id_light** | [**AccountIdLight**](AccountIdLight.md) |  | [required] |

### Return type

[**crate::models::LoginResult**](LoginResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_register

> crate::models::AccountIdLight post_register()
Register new account. Returns new account ID which is UUID.

Register new account. Returns new account ID which is UUID.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::AccountIdLight**](AccountIdLight.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_sign_in_with_login

> crate::models::ApiKey post_sign_in_with_login(sign_in_with_login_info)
Start new session with sign in with Apple or Google. Creates new account if

Start new session with sign in with Apple or Google. Creates new account if it does not exists.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sign_in_with_login_info** | [**SignInWithLoginInfo**](SignInWithLoginInfo.md) |  | [required] |

### Return type

[**crate::models::ApiKey**](ApiKey.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_setting_profile_visiblity

> put_setting_profile_visiblity(boolean_setting)
Update profile visiblity value.

Update profile visiblity value.  This will check that the first image moderation request has been moderated before this turns the profile public.  Sets capablity `view_public_profiles` on or off depending on the value.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**boolean_setting** | [**BooleanSetting**](BooleanSetting.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

