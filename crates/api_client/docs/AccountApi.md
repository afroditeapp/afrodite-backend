# \AccountApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_cancel_deletion**](AccountApi.md#delete_cancel_deletion) | **DELETE** /account_api/delete | Cancel account deletion.
[**get_account_data**](AccountApi.md#get_account_data) | **GET** /account_api/account_data | Get changeable user information to account.
[**get_account_setup**](AccountApi.md#get_account_setup) | **GET** /account_api/account_setup | Get non-changeable user information to account.
[**get_account_state**](AccountApi.md#get_account_state) | **GET** /account_api/state | Get current account state.
[**get_deletion_status**](AccountApi.md#get_deletion_status) | **GET** /account_api/delete | Get deletion status.
[**get_latest_birthdate**](AccountApi.md#get_latest_birthdate) | **GET** /account_api/latest_birthdate | 
[**post_account_data**](AccountApi.md#post_account_data) | **POST** /account_api/account_data | Set changeable user information to account.
[**post_account_setup**](AccountApi.md#post_account_setup) | **POST** /account_api/account_setup | Setup non-changeable user information during `initial setup` state.
[**post_complete_setup**](AccountApi.md#post_complete_setup) | **POST** /account_api/complete_setup | Complete initial setup.
[**post_delete**](AccountApi.md#post_delete) | **PUT** /account_api/delete | Delete account.
[**post_demo_mode_accessible_accounts**](AccountApi.md#post_demo_mode_accessible_accounts) | **POST** /account_api/demo_mode_accessible_accounts | Get demo account's available accounts.
[**post_demo_mode_confirm_login**](AccountApi.md#post_demo_mode_confirm_login) | **POST** /account_api/demo_mode_confirm_login | 
[**post_demo_mode_login**](AccountApi.md#post_demo_mode_login) | **POST** /account_api/demo_mode_login | Access demo mode, which allows accessing all or specific accounts
[**post_demo_mode_login_to_account**](AccountApi.md#post_demo_mode_login_to_account) | **POST** /account_api/demo_mode_login_to_account | 
[**post_demo_mode_register_account**](AccountApi.md#post_demo_mode_register_account) | **POST** /account_api/demo_mode_register_account | 
[**post_sign_in_with_login**](AccountApi.md#post_sign_in_with_login) | **POST** /account_api/sign_in_with_login | Start new session with sign in with Apple or Google. Creates new account if
[**put_setting_profile_visiblity**](AccountApi.md#put_setting_profile_visiblity) | **PUT** /account_api/settings/profile_visibility | Update current or pending profile visiblity value.
[**put_setting_unlimited_likes**](AccountApi.md#put_setting_unlimited_likes) | **PUT** /account_api/settings/unlimited_likes | 



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

> models::AccountData get_account_data()
Get changeable user information to account.

Get changeable user information to account.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::AccountData**](AccountData.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_account_setup

> models::AccountSetup get_account_setup()
Get non-changeable user information to account.

Get non-changeable user information to account.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::AccountSetup**](AccountSetup.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_account_state

> models::Account get_account_state()
Get current account state.

Get current account state.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::Account**](Account.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_deletion_status

> models::DeleteStatus get_deletion_status()
Get deletion status.

Get deletion status.  Get information when account will be really deleted.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::DeleteStatus**](DeleteStatus.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_latest_birthdate

> models::LatestBirthdate get_latest_birthdate()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::LatestBirthdate**](LatestBirthdate.md)

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

> post_account_setup(set_account_setup)
Setup non-changeable user information during `initial setup` state.

Setup non-changeable user information during `initial setup` state.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**set_account_setup** | [**SetAccountSetup**](SetAccountSetup.md) |  | [required] |

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

Complete initial setup.  Requirements: - Account must be in `InitialSetup` state. - Account must have a valid AccountSetup info set. - Account must have a moderation request. - The current or pending security image of the account is in the request. - The current or pending first profile image of the account is in the request. 

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


## post_demo_mode_accessible_accounts

> Vec<models::AccessibleAccount> post_demo_mode_accessible_accounts(demo_mode_token)
Get demo account's available accounts.

Get demo account's available accounts.  This path is using HTTP POST because there is JSON in the request body.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**demo_mode_token** | [**DemoModeToken**](DemoModeToken.md) |  | [required] |

### Return type

[**Vec<models::AccessibleAccount>**](AccessibleAccount.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_demo_mode_confirm_login

> models::DemoModeConfirmLoginResult post_demo_mode_confirm_login(demo_mode_confirm_login)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**demo_mode_confirm_login** | [**DemoModeConfirmLogin**](DemoModeConfirmLogin.md) |  | [required] |

### Return type

[**models::DemoModeConfirmLoginResult**](DemoModeConfirmLoginResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_demo_mode_login

> models::DemoModeLoginResult post_demo_mode_login(demo_mode_password)
Access demo mode, which allows accessing all or specific accounts

Access demo mode, which allows accessing all or specific accounts depending on the server configuration.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**demo_mode_password** | [**DemoModePassword**](DemoModePassword.md) |  | [required] |

### Return type

[**models::DemoModeLoginResult**](DemoModeLoginResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_demo_mode_login_to_account

> models::LoginResult post_demo_mode_login_to_account(demo_mode_login_to_account)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**demo_mode_login_to_account** | [**DemoModeLoginToAccount**](DemoModeLoginToAccount.md) |  | [required] |

### Return type

[**models::LoginResult**](LoginResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_demo_mode_register_account

> models::AccountId post_demo_mode_register_account(demo_mode_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**demo_mode_token** | [**DemoModeToken**](DemoModeToken.md) |  | [required] |

### Return type

[**models::AccountId**](AccountId.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_sign_in_with_login

> models::LoginResult post_sign_in_with_login(sign_in_with_login_info)
Start new session with sign in with Apple or Google. Creates new account if

Start new session with sign in with Apple or Google. Creates new account if it does not exists.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sign_in_with_login_info** | [**SignInWithLoginInfo**](SignInWithLoginInfo.md) |  | [required] |

### Return type

[**models::LoginResult**](LoginResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_setting_profile_visiblity

> put_setting_profile_visiblity(boolean_setting)
Update current or pending profile visiblity value.

Update current or pending profile visiblity value.  NOTE: Client uses this in initial setup.

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


## put_setting_unlimited_likes

> put_setting_unlimited_likes(boolean_setting)


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

