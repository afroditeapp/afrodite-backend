# \ChatApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_like**](ChatApi.md#delete_like) | **DELETE** /chat_api/delete_like | Delete sent like.
[**delete_pending_messages**](ChatApi.md#delete_pending_messages) | **DELETE** /chat_api/pending_messages | Delete list of pending messages
[**get_matches**](ChatApi.md#get_matches) | **GET** /chat_api/matches | Get matches
[**get_message_number_of_latest_viewed_message**](ChatApi.md#get_message_number_of_latest_viewed_message) | **GET** /chat_api/message_number_of_latest_viewed_message | Get message number of the most recent message that the recipient has viewed.
[**get_pending_messages**](ChatApi.md#get_pending_messages) | **GET** /chat_api/pending_messages | Get list of pending messages
[**get_received_blocks**](ChatApi.md#get_received_blocks) | **GET** /chat_api/received_blocks | Get list of received blocks
[**get_received_likes**](ChatApi.md#get_received_likes) | **GET** /chat_api/received_likes | Get received likes.
[**get_sent_blocks**](ChatApi.md#get_sent_blocks) | **GET** /chat_api/sent_blocks | Get list of sent blocks
[**get_sent_likes**](ChatApi.md#get_sent_likes) | **GET** /chat_api/sent_likes | Get sent likes.
[**post_block_profile**](ChatApi.md#post_block_profile) | **POST** /chat_api/block_profile | Block profile
[**post_get_pending_notification**](ChatApi.md#post_get_pending_notification) | **POST** /chat_api/get_pending_notification | Get pending notification and reset pending notification.
[**post_message_number_of_latest_viewed_message**](ChatApi.md#post_message_number_of_latest_viewed_message) | **POST** /chat_api/message_number_of_latest_viewed_message | Update message number of the most recent message that the recipient has viewed.
[**post_send_like**](ChatApi.md#post_send_like) | **POST** /chat_api/send_like | Send a like to some account. If both will like each other, then
[**post_send_message**](ChatApi.md#post_send_message) | **POST** /chat_api/send_message | Send message to a match
[**post_set_device_token**](ChatApi.md#post_set_device_token) | **POST** /chat_api/set_device_token | 
[**post_unblock_profile**](ChatApi.md#post_unblock_profile) | **POST** /chat_api/unblock_profile | Unblock profile



## delete_like

> delete_like(account_id)
Delete sent like.

Delete sent like.  Delete will not work if profile is a match.

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


## delete_pending_messages

> delete_pending_messages(pending_message_delete_list)
Delete list of pending messages

Delete list of pending messages

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**pending_message_delete_list** | [**PendingMessageDeleteList**](PendingMessageDeleteList.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_matches

> crate::models::MatchesPage get_matches()
Get matches

Get matches

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::MatchesPage**](MatchesPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_message_number_of_latest_viewed_message

> crate::models::MessageNumber get_message_number_of_latest_viewed_message(account_id)
Get message number of the most recent message that the recipient has viewed.

Get message number of the most recent message that the recipient has viewed.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | [**AccountId**](AccountId.md) |  | [required] |

### Return type

[**crate::models::MessageNumber**](MessageNumber.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_pending_messages

> crate::models::PendingMessagesPage get_pending_messages()
Get list of pending messages

Get list of pending messages

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::PendingMessagesPage**](PendingMessagesPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_received_blocks

> crate::models::ReceivedBlocksPage get_received_blocks()
Get list of received blocks

Get list of received blocks

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ReceivedBlocksPage**](ReceivedBlocksPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_received_likes

> crate::models::ReceivedLikesPage get_received_likes()
Get received likes.

Get received likes.  Profile will not be returned if: - Profile is blocked - Profile is a match

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::ReceivedLikesPage**](ReceivedLikesPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_sent_blocks

> crate::models::SentBlocksPage get_sent_blocks()
Get list of sent blocks

Get list of sent blocks

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::SentBlocksPage**](SentBlocksPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_sent_likes

> crate::models::SentLikesPage get_sent_likes()
Get sent likes.

Get sent likes.  Profile will not be returned if:  - Profile is hidden (not public) - Profile is blocked - Profile is a match

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::SentLikesPage**](SentLikesPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_block_profile

> post_block_profile(account_id)
Block profile

Block profile

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


## post_get_pending_notification

> crate::models::PendingNotificationWithData post_get_pending_notification(pending_notification_token)
Get pending notification and reset pending notification.

Get pending notification and reset pending notification.  Requesting this route is always valid to avoid figuring out device token values more easily.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**pending_notification_token** | [**PendingNotificationToken**](PendingNotificationToken.md) |  | [required] |

### Return type

[**crate::models::PendingNotificationWithData**](PendingNotificationWithData.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_message_number_of_latest_viewed_message

> post_message_number_of_latest_viewed_message(update_message_view_status)
Update message number of the most recent message that the recipient has viewed.

Update message number of the most recent message that the recipient has viewed.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**update_message_view_status** | [**UpdateMessageViewStatus**](UpdateMessageViewStatus.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_send_like

> post_send_like(account_id)
Send a like to some account. If both will like each other, then

Send a like to some account. If both will like each other, then the accounts will be a match.

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


## post_send_message

> post_send_message(send_message_to_account)
Send message to a match

Send message to a match

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**send_message_to_account** | [**SendMessageToAccount**](SendMessageToAccount.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_set_device_token

> crate::models::PendingNotificationToken post_set_device_token(fcm_device_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**fcm_device_token** | [**FcmDeviceToken**](FcmDeviceToken.md) |  | [required] |

### Return type

[**crate::models::PendingNotificationToken**](PendingNotificationToken.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_unblock_profile

> post_unblock_profile(account_id)
Unblock profile

Unblock profile

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

