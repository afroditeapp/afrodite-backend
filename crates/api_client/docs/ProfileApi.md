# \ProfileApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_favorite_profile**](ProfileApi.md#delete_favorite_profile) | **DELETE** /yD1PtVhVvdk-usEran42JmCTFVQ | Delete favorite profile
[**get_available_profile_attributes**](ProfileApi.md#get_available_profile_attributes) | **GET** /_lqy4YCINbw_RCxebKLGXdDq2AM | Get info what profile attributes server supports.
[**get_favorite_profiles**](ProfileApi.md#get_favorite_profiles) | **GET** /Oep5nM7bWqTdRfRoULt-_FTkKJQ | Get list of all favorite profiles.
[**get_initial_profile_age_info**](ProfileApi.md#get_initial_profile_age_info) | **GET** /NWOWjOlm6oTYsXiPsbhBDgknan0 | Get initial profile age information which can be used for calculating
[**get_location**](ProfileApi.md#get_location) | **GET** /lf5KMD9dBSVuaVcwjm4TB0d7bfY | Get location for account which makes this request.
[**get_my_profile**](ProfileApi.md#get_my_profile) | **GET** /iu25rmmvUzADXhW5SsP_DBGY2_w | Get my profile
[**get_profile**](ProfileApi.md#get_profile) | **GET** /5i55ZcY0jIPD7B6pyyridKY0j0Q/{aid} | Get account's current profile.
[**get_profile_attribute_filters**](ProfileApi.md#get_profile_attribute_filters) | **GET** /AL531AoIDRcTSWC-pdxcexf6tOM | Get current profile attribute filter values.
[**get_profile_from_database_debug_mode_benchmark**](ProfileApi.md#get_profile_from_database_debug_mode_benchmark) | **GET** /XDTSz35S_5tOKIsSpDITOc46MR4/{aid} | Get account's current profile from database. Debug mode must be enabled
[**get_search_age_range**](ProfileApi.md#get_search_age_range) | **GET** /xTy-zcnl0LQlfPKQalAEnWQQ-rw | Get account's current search age range
[**get_search_groups**](ProfileApi.md#get_search_groups) | **GET** /p1KA-sqKKtU3FHvUqYRZnQgj7RQ | Get account's current search groups
[**post_favorite_profile**](ProfileApi.md#post_favorite_profile) | **POST** /yD1PtVhVvdk-usEran42JmCTFVQ | Add new favorite profile
[**post_get_next_profile_page**](ProfileApi.md#post_get_next_profile_page) | **POST** /_XRgLHtmWtbgW3ZAlgfTH5bs6bE | Post (updates iterator) to get next page of profile list.
[**post_profile**](ProfileApi.md#post_profile) | **POST** /5i55ZcY0jIPD7B6pyyridKY0j0Q | Update profile information.
[**post_profile_attribute_filters**](ProfileApi.md#post_profile_attribute_filters) | **POST** /AL531AoIDRcTSWC-pdxcexf6tOM | Set profile attribute filter values.
[**post_profile_to_database_debug_mode_benchmark**](ProfileApi.md#post_profile_to_database_debug_mode_benchmark) | **POST** /XDTSz35S_5tOKIsSpDITOc46MR4 | Post account's current profile directly to database. Debug mode must be enabled
[**post_reset_profile_paging**](ProfileApi.md#post_reset_profile_paging) | **POST** /uUYIl9C8DoXwTj1icArj0S4RTFI | Reset profile paging.
[**post_search_age_range**](ProfileApi.md#post_search_age_range) | **POST** /xTy-zcnl0LQlfPKQalAEnWQQ-rw | Set account's current search age range
[**post_search_groups**](ProfileApi.md#post_search_groups) | **POST** /p1KA-sqKKtU3FHvUqYRZnQgj7RQ | Set account's current search groups
[**put_location**](ProfileApi.md#put_location) | **PUT** /lf5KMD9dBSVuaVcwjm4TB0d7bfY | Update location for account which makes this request.



## delete_favorite_profile

> delete_favorite_profile(account_id)
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

> models::AvailableProfileAttributes get_available_profile_attributes()
Get info what profile attributes server supports.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::AvailableProfileAttributes**](AvailableProfileAttributes.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_favorite_profiles

> models::FavoriteProfilesPage get_favorite_profiles()
Get list of all favorite profiles.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::FavoriteProfilesPage**](FavoriteProfilesPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_initial_profile_age_info

> models::GetInitialProfileAgeInfoResult get_initial_profile_age_info()
Get initial profile age information which can be used for calculating

current accepted profile ages.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetInitialProfileAgeInfoResult**](GetInitialProfileAgeInfoResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_location

> models::Location get_location()
Get location for account which makes this request.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::Location**](Location.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_my_profile

> models::GetMyProfileResult get_my_profile()
Get my profile

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetMyProfileResult**](GetMyProfileResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_profile

> models::GetProfileResult get_profile(aid, v, is_match)
Get account's current profile.

Response includes version UUID which can be used for caching.  # Access  ## Own profile Unrestricted access.  ## Public other profiles Normal account state required.  ## Private other profiles If the profile is a match, then the profile can be accessed if query parameter `is_match` is set to `true`.  If the profile is not a match, then capability `admin_view_all_profiles` is required.  # Microservice notes If account feature is set as external service then cached capability information from account service is used for access checks.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**aid** | **uuid::Uuid** |  | [required] |
**v** | Option<**uuid::Uuid**> | Profile version UUID |  |
**is_match** | Option<**bool**> | If requested profile is not public, allow getting the profile data if the requested profile is a match. |  |

### Return type

[**models::GetProfileResult**](GetProfileResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_profile_attribute_filters

> models::ProfileAttributeFilterList get_profile_attribute_filters()
Get current profile attribute filter values.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ProfileAttributeFilterList**](ProfileAttributeFilterList.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_profile_from_database_debug_mode_benchmark

> models::Profile get_profile_from_database_debug_mode_benchmark(aid)
Get account's current profile from database. Debug mode must be enabled

that route can be used.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**aid** | **uuid::Uuid** |  | [required] |

### Return type

[**models::Profile**](Profile.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_search_age_range

> models::ProfileSearchAgeRange get_search_age_range()
Get account's current search age range

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ProfileSearchAgeRange**](ProfileSearchAgeRange.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_search_groups

> models::SearchGroups get_search_groups()
Get account's current search groups

(gender and what gender user is looking for)

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::SearchGroups**](SearchGroups.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_favorite_profile

> post_favorite_profile(account_id)
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

> models::ProfilePage post_get_next_profile_page(iterator_session_id)
Post (updates iterator) to get next page of profile list.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**iterator_session_id** | [**IteratorSessionId**](IteratorSessionId.md) |  | [required] |

### Return type

[**models::ProfilePage**](ProfilePage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_profile

> post_profile(profile_update)
Update profile information.

Writes the profile to the database only if it is changed.  WebSocket event about profile change will not be emitted. The event is emitted only from server side profile updates.  # Requirements - Profile attributes must be valid. - Profile text must be empty. - Profile name changes are only possible when initial setup is ongoing. - Profile age must match with currently valid age range. The first min value for the age range is the age at the initial setup. The second min and max value is calculated using the following algorithm: - The initial age (initialAge) is paired with the year of initial setup completed (initialSetupYear). - Year difference (yearDifference = currentYear - initialSetupYear) is used for changing the range min and max. - Min value: initialAge + yearDifference - 1. - Max value: initialAge + yearDifference + 1.  TODO: string lenght validation, limit saving new profiles TODO: return the new proifle. Edit: is this really needed?

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

that route can be used.

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

> models::IteratorSessionId post_reset_profile_paging()
Reset profile paging.

After this request getting next profiles will continue from the nearest profiles.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::IteratorSessionId**](IteratorSessionId.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_search_age_range

> post_search_age_range(profile_search_age_range)
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

(gender and what gender user is looking for)

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

