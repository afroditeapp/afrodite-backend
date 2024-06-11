# \MediaApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_content**](MediaApi.md#delete_content) | **DELETE** /media_api/content/{account_id}/{content_id} | Delete content data. Content can be removed after specific time has passed
[**delete_moderation_request**](MediaApi.md#delete_moderation_request) | **DELETE** /media_api/moderation/request | Delete current moderation request which is not yet in moderation.
[**delete_pending_security_content_info**](MediaApi.md#delete_pending_security_content_info) | **DELETE** /media_api/pending_security_content_info | Delete pending security content for current account.
[**get_all_account_media_content**](MediaApi.md#get_all_account_media_content) | **GET** /media_api/all_account_media_content/{account_id} | Get list of all media content on the server for one account.
[**get_content**](MediaApi.md#get_content) | **GET** /media_api/content/{account_id}/{content_id} | Get content data
[**get_content_slot_state**](MediaApi.md#get_content_slot_state) | **GET** /media_api/content_slot/{slot_id} | Get state of content slot.
[**get_map_tile**](MediaApi.md#get_map_tile) | **GET** /media_api/map_tile/{z}/{x}/{y} | Get map tile PNG file.
[**get_moderation_request**](MediaApi.md#get_moderation_request) | **GET** /media_api/moderation/request | Get current moderation request.
[**get_pending_profile_content_info**](MediaApi.md#get_pending_profile_content_info) | **GET** /media_api/pending_profile_content_info/{account_id} | Get pending profile content for selected profile
[**get_pending_security_content_info**](MediaApi.md#get_pending_security_content_info) | **GET** /media_api/pending_security_content_info/{account_id} | Get pending security content for selected profile.
[**get_profile_content_info**](MediaApi.md#get_profile_content_info) | **GET** /media_api/profile_content_info/{account_id} | Get current profile content for selected profile.
[**get_security_content_info**](MediaApi.md#get_security_content_info) | **GET** /media_api/security_content_info/{account_id} | Get current security content for selected profile.
[**put_content_to_content_slot**](MediaApi.md#put_content_to_content_slot) | **PUT** /media_api/content_slot/{slot_id} | Set content to content processing slot.
[**put_moderation_request**](MediaApi.md#put_moderation_request) | **PUT** /media_api/moderation/request | Create new or override old moderation request.
[**put_pending_profile_content**](MediaApi.md#put_pending_profile_content) | **PUT** /media_api/pending_profile_content | Set new pending profile content for current account.
[**put_pending_security_content_info**](MediaApi.md#put_pending_security_content_info) | **PUT** /media_api/pending_security_content_info | Set pending security content for current account.
[**put_profile_content**](MediaApi.md#put_profile_content) | **PUT** /media_api/profile_content | Set new profile content for current account.
[**put_security_content_info**](MediaApi.md#put_security_content_info) | **PUT** /media_api/security_content_info | Set current security content content for current account.



## delete_content

> delete_content(account_id, content_id)
Delete content data. Content can be removed after specific time has passed

Delete content data. Content can be removed after specific time has passed since removing all usage from it (content is not a security image or profile content).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |
**content_id** | **uuid::Uuid** |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_moderation_request

> delete_moderation_request()
Delete current moderation request which is not yet in moderation.

Delete current moderation request which is not yet in moderation.

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


## delete_pending_security_content_info

> delete_pending_security_content_info()
Delete pending security content for current account.

Delete pending security content for current account. Server will not change the security content when next moderation request is moderated as accepted.

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


## get_all_account_media_content

> crate::models::AccountContent get_all_account_media_content(account_id)
Get list of all media content on the server for one account.

Get list of all media content on the server for one account.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |

### Return type

[**crate::models::AccountContent**](AccountContent.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_content

> std::path::PathBuf get_content(account_id, content_id, is_match)
Get content data

Get content data  # Access  ## Own content Unrestricted access.  ## Public other content Normal account state required.  ## Private other content If owner of the requested content is a match and the requested content is in current profile content, then the requested content can be accessed if query parameter `is_match` is set to `true`.  If the previous is not true, then capability `admin_view_all_profiles` or `admin_moderate_images` is required. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |
**content_id** | **uuid::Uuid** |  | [required] |
**is_match** | Option<**bool**> | If false media content access is allowed when profile is set as public. If true media content access is allowed when users are a match. |  |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_content_slot_state

> crate::models::ContentProcessingState get_content_slot_state(slot_id)
Get state of content slot.

Get state of content slot.  Slots from 0 to 6 are available. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**slot_id** | **i32** |  | [required] |

### Return type

[**crate::models::ContentProcessingState**](ContentProcessingState.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_map_tile

> std::path::PathBuf get_map_tile(z, x, y)
Get map tile PNG file.

Get map tile PNG file.  Returns a .png even if the URL does not have it.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**z** | **i32** |  | [required] |
**x** | **i32** |  | [required] |
**y** | **String** |  | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: image/png

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_moderation_request

> crate::models::CurrentModerationRequest get_moderation_request()
Get current moderation request.

Get current moderation request. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::CurrentModerationRequest**](CurrentModerationRequest.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_pending_profile_content_info

> crate::models::PendingProfileContent get_pending_profile_content_info(account_id)
Get pending profile content for selected profile

Get pending profile content for selected profile

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |

### Return type

[**crate::models::PendingProfileContent**](PendingProfileContent.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_pending_security_content_info

> crate::models::PendingSecurityContent get_pending_security_content_info(account_id)
Get pending security content for selected profile.

Get pending security content for selected profile.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |

### Return type

[**crate::models::PendingSecurityContent**](PendingSecurityContent.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_profile_content_info

> crate::models::GetProfileContentResult get_profile_content_info(account_id, version, is_match)
Get current profile content for selected profile.

Get current profile content for selected profile.  # Access  ## Own profile Unrestricted access.  ## Other profiles Normal account state required.  ## Private other profiles If the profile is a match, then the profile can be accessed if query parameter `is_match` is set to `true`.  If the profile is not a match, then capability `admin_view_all_profiles` is required.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |
**version** | Option<**uuid::Uuid**> |  |  |
**is_match** | Option<**bool**> | If false profile content access is allowed when profile is set as public. If true profile content access is allowed when users are a match. |  |

### Return type

[**crate::models::GetProfileContentResult**](GetProfileContentResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_security_content_info

> crate::models::SecurityContent get_security_content_info(account_id)
Get current security content for selected profile.

Get current security content for selected profile.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | **uuid::Uuid** |  | [required] |

### Return type

[**crate::models::SecurityContent**](SecurityContent.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_content_to_content_slot

> crate::models::ContentProcessingId put_content_to_content_slot(slot_id, secure_capture, content_type, body)
Set content to content processing slot.

Set content to content processing slot. Processing ID will be returned and processing of the content will begin. Events about the content processing will be sent to the client.  The state of the processing can be also queired. The querying is required to receive the content ID.  Slots from 0 to 6 are available.  One account can only have one content in upload or processing state. New upload might potentially delete the previous if processing of it is not complete.  Content processing will fail if image content resolution width or height value is less than 512. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**slot_id** | **i32** |  | [required] |
**secure_capture** | **bool** | Client captured this content. | [required] |
**content_type** | [**MediaContentType**](.md) |  | [required] |
**body** | **std::path::PathBuf** |  | [required] |

### Return type

[**crate::models::ContentProcessingId**](ContentProcessingId.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: image/jpeg
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_moderation_request

> put_moderation_request(moderation_request_content)
Create new or override old moderation request.

Create new or override old moderation request.  Make sure that moderation request has content IDs which points to your own image slots. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**moderation_request_content** | [**ModerationRequestContent**](ModerationRequestContent.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_pending_profile_content

> put_pending_profile_content(set_profile_content)
Set new pending profile content for current account.

Set new pending profile content for current account. Server will switch to pending content when next moderation request is accepted.  # Restrictions - All content must not be moderated as rejected. - All content must be owned by the account. - All content must be images.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**set_profile_content** | [**SetProfileContent**](SetProfileContent.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_pending_security_content_info

> put_pending_security_content_info(content_id)
Set pending security content for current account.

Set pending security content for current account.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**content_id** | [**ContentId**](ContentId.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_profile_content

> put_profile_content(set_profile_content)
Set new profile content for current account.

Set new profile content for current account.  # Restrictions - All content must be moderated as accepted. - All content must be owned by the account. - All content must be images.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**set_profile_content** | [**SetProfileContent**](SetProfileContent.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_security_content_info

> put_security_content_info(content_id)
Set current security content content for current account.

Set current security content content for current account.  # Restrictions - The content must be moderated as accepted. - The content must be owned by the account. - The content must be an image. - The content must be captured by client.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**content_id** | [**ContentId**](ContentId.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

