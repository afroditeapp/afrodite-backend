# \ProfileApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_favorite_profile**](ProfileApi.md#delete_favorite_profile) | **DELETE** /profile_api/favorite_profile | Delete favorite profile
[**get_available_profile_attributes**](ProfileApi.md#get_available_profile_attributes) | **GET** /profile_api/available_profile_attributes | Get info what profile attributes server supports.
[**get_favorite_profiles**](ProfileApi.md#get_favorite_profiles) | **GET** /profile_api/favorite_profiles | Get list of all favorite profiles.
[**get_location**](ProfileApi.md#get_location) | **GET** /profile_api/location | Get location for account which makes this request.
[**get_profile**](ProfileApi.md#get_profile) | **GET** /profile_api/profile/{account_id} | Get account's current profile.
[**get_profile_attribute_filters**](ProfileApi.md#get_profile_attribute_filters) | **GET** /profile_api/profile_attribute_filters | Get current profile attribute filter values.
[**get_profile_from_database_debug_mode_benchmark**](ProfileApi.md#get_profile_from_database_debug_mode_benchmark) | **GET** /profile_api/benchmark/profile/{account_id} | Get account's current profile from database. Debug mode must be enabled
[**get_search_age_range**](ProfileApi.md#get_search_age_range) | **GET** /profile_api/search_age_range | Get account's current search age range
[**get_search_groups**](ProfileApi.md#get_search_groups) | **GET** /profile_api/search_groups | Get account's current search groups
[**post_favorite_profile**](ProfileApi.md#post_favorite_profile) | **POST** /profile_api/favorite_profile | Add new favorite profile
[**post_get_next_profile_page**](ProfileApi.md#post_get_next_profile_page) | **POST** /profile_api/page/next | Post (updates iterator) to get next page of profile list.
[**post_profile**](ProfileApi.md#post_profile) | **POST** /profile_api/profile | Update profile information.
[**post_profile_attribute_filters**](ProfileApi.md#post_profile_attribute_filters) | **POST** /profile_api/profile_attribute_filters | Set profile attribute filter values.
[**post_profile_to_database_debug_mode_benchmark**](ProfileApi.md#post_profile_to_database_debug_mode_benchmark) | **POST** /profile_api/benchmark/profile | Post account's current profile directly to database. Debug mode must be enabled
[**post_reset_profile_paging**](ProfileApi.md#post_reset_profile_paging) | **POST** /profile_api/page/reset | Reset profile paging.
[**post_search_age_range**](ProfileApi.md#post_search_age_range) | **POST** /profile_api/search_age_range | Set account's current search age range
[**post_search_groups**](ProfileApi.md#post_search_groups) | **POST** /profile_api/search_groups | Set account's current search groups
[**put_location**](ProfileApi.md#put_location) | **PUT** /profile_api/location | Update location for account which makes this request.



## delete_favorite_profile

> delete_favorite_profile(account_id)
Delete favorite profile

Delete favorite profile

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | [**AccountId**](AccountId.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_available_profile_attributes

> crate::models::AvailableProfileAttributes get_available_profile_attributes()
Get info what profile attributes server supports.

Get info what profile attributes server supports.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::AvailableProfileAttributes**](AvailableProfileAttributes.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_favorite_profiles

> crate::models::FavoriteProfilesPage get_favorite_profiles()
Get list of all favorite profiles.

Get list of all favorite profiles.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::FavoriteProfilesPage**](FavoriteProfilesPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_location

> crate::models::Location get_location()
Get location for account which makes this request.

Get location for account which makes this request.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::Location**](Location.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_profile

> crate::models::GetProfileResult get_profile(account_id, version, is_match)
Get account's current profile.

Get account's current profile.  Response includes version UUID which can be used for caching.  # Access  ## Own profile Unrestricted access.  ## Public other profiles Normal account state required.  ## Private other profiles If the profile is a match, then the profile can be accessed if query parameter `is_match` is set to `true`.  If the profile is not a match, then capability `admin_view_all_profiles` is required.  # Microservice notes If account feature is set as external service then cached capability information from account service is used for access checks.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |
**version** | Option<**uuid::Uuid**> | Profile version UUID |  |
**is_match** | Option<**bool**> | If requested profile is not public, allow getting the profile data if the requested profile is a match. |  |

### Return type

[**crate::models::GetProfileResult**](GetProfileResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_profile_attribute_filters

> crate::models::ProfileAttributeFilterList get_profile_attribute_filters()
Get current profile attribute filter values.

Get current profile attribute filter values.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ProfileAttributeFilterList**](ProfileAttributeFilterList.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_profile_from_database_debug_mode_benchmark

> crate::models::Profile get_profile_from_database_debug_mode_benchmark(account_id)
Get account's current profile from database. Debug mode must be enabled

Get account's current profile from database. Debug mode must be enabled that route can be used.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |

### Return type

[**crate::models::Profile**](Profile.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_search_age_range

> crate::models::ProfileSearchAgeRange get_search_age_range()
Get account's current search age range

Get account's current search age range

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ProfileSearchAgeRange**](ProfileSearchAgeRange.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_search_groups

> crate::models::SearchGroups get_search_groups()
Get account's current search groups

Get account's current search groups (gender and what gender user is looking for)

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::SearchGroups**](SearchGroups.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_favorite_profile

> post_favorite_profile(account_id)
Add new favorite profile

Add new favorite profile

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | [**AccountId**](AccountId.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_get_next_profile_page

> crate::models::ProfilePage post_get_next_profile_page()
Post (updates iterator) to get next page of profile list.

Post (updates iterator) to get next page of profile list.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ProfilePage**](ProfilePage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_profile

> post_profile(profile_update)
Update profile information.

Update profile information.  Writes the profile to the database only if it is changed.  TODO: string lenght validation, limit saving new profiles TODO: return the new proifle. Edit: is this really needed?

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**profile_update** | [**ProfileUpdate**](ProfileUpdate.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_profile_attribute_filters

> post_profile_attribute_filters(profile_attribute_filter_list_update)
Set profile attribute filter values.

Set profile attribute filter values.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**profile_attribute_filter_list_update** | [**ProfileAttributeFilterListUpdate**](ProfileAttributeFilterListUpdate.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_profile_to_database_debug_mode_benchmark

> post_profile_to_database_debug_mode_benchmark(profile_update)
Post account's current profile directly to database. Debug mode must be enabled

Post account's current profile directly to database. Debug mode must be enabled that route can be used.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**profile_update** | [**ProfileUpdate**](ProfileUpdate.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_reset_profile_paging

> post_reset_profile_paging()
Reset profile paging.

Reset profile paging.  After this request getting next profiles will continue from the nearest profiles.

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


## post_search_age_range

> post_search_age_range(profile_search_age_range)
Set account's current search age range

Set account's current search age range

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**profile_search_age_range** | [**ProfileSearchAgeRange**](ProfileSearchAgeRange.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_search_groups

> post_search_groups(search_groups)
Set account's current search groups

Set account's current search groups (gender and what gender user is looking for)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**search_groups** | [**SearchGroups**](SearchGroups.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_location

> put_location(location)
Update location for account which makes this request.

Update location for account which makes this request.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**location** | [**Location**](Location.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

