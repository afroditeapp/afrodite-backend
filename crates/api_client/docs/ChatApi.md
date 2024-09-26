# \ChatApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_like**](ChatApi.md#delete_like) | **DELETE** /O3DZlGOjDYbQ8PlCorta0onQDLI | Delete sent like.
[**get_matches**](ChatApi.md#get_matches) | **GET** /kzySqAe9qYi69MoCBnFGKdn76-Q | Get matches
[**get_message_number_of_latest_viewed_message**](ChatApi.md#get_message_number_of_latest_viewed_message) | **GET** /gas7m77c7kw7N7TKyMQVzUKy3AQ | Get message number of the most recent message that the recipient has viewed.
[**get_pending_messages**](ChatApi.md#get_pending_messages) | **GET** /7sKe87sefWrLYS0JvbPS10_F8oc | Get list of pending messages.
[**get_public_key**](ChatApi.md#get_public_key) | **GET** /e-r4VrqWJD1kIttg1McD9kv5o0k/{aid} | Get current public key of some account
[**get_received_blocks**](ChatApi.md#get_received_blocks) | **GET** /hjP9LovH2kBxnbwWKSyVgFL4o58 | Get list of received blocks
[**get_sent_blocks**](ChatApi.md#get_sent_blocks) | **GET** /3qT3qSKKzXHjo8LGEphwMLF8vjk | Get list of sent blocks
[**get_sent_likes**](ChatApi.md#get_sent_likes) | **GET** /pTybb424uGsXvCyOLljsPujVe5Y | Get sent likes.
[**get_sent_message_ids**](ChatApi.md#get_sent_message_ids) | **GET** /-sDy1a8MS72uNy3UUtX9K8-wYWU | 
[**post_add_receiver_acknowledgement**](ChatApi.md#post_add_receiver_acknowledgement) | **POST** /PBreZU5Cmo7tTtNMMb58yN_xFZ8 | 
[**post_add_sender_acknowledgement**](ChatApi.md#post_add_sender_acknowledgement) | **POST** /E-yVIcGOLJyZ7nsT_Lh4KPCRkQg | 
[**post_block_profile**](ChatApi.md#post_block_profile) | **POST** /MpWSY01lXj7KaDK1KCNHLWRg9k4 | Block profile
[**post_get_new_received_likes_count**](ChatApi.md#post_get_new_received_likes_count) | **POST** /dCPla4TZep6KONk57U2J7p7s6jw | 
[**post_get_next_received_likes_page**](ChatApi.md#post_get_next_received_likes_page) | **POST** /eEB4pq6DGUYlMVAYwPCm2RT5HP0 | Update received likes iterator and get next page
[**post_get_pending_notification**](ChatApi.md#post_get_pending_notification) | **POST** /MhQXhJMKgrUh0s95FueOgalQg-o | Get pending notification and reset pending notification.
[**post_message_number_of_latest_viewed_message**](ChatApi.md#post_message_number_of_latest_viewed_message) | **POST** /gas7m77c7kw7N7TKyMQVzUKy3AQ | Update message number of the most recent message that the recipient has viewed.
[**post_public_key**](ChatApi.md#post_public_key) | **POST** /e-r4VrqWJD1kIttg1McD9kv5o0k | Replace current public key with a new public key.
[**post_reset_received_likes_paging**](ChatApi.md#post_reset_received_likes_paging) | **POST** /B75BRIylLV-JmwoB4YiOYSlyO-A | 
[**post_send_like**](ChatApi.md#post_send_like) | **POST** /sXq6ko76GtT7DuNXnkTTtFL6isY | Send a like to some account. If both will like each other, then
[**post_send_message**](ChatApi.md#post_send_message) | **POST** /YEFESgzw0YxQUETcUmnmfWCaF1g | Send message to a match.
[**post_set_device_token**](ChatApi.md#post_set_device_token) | **POST** /CBoGGZ4HDW0REbM6SxasDCvXJNM | 
[**post_unblock_profile**](ChatApi.md#post_unblock_profile) | **POST** /j2Ofh-WeAFmjCQqO_AyHIM1eZEo | Unblock profile



## delete_like

> models::LimitedActionResult delete_like(account_id)
Delete sent like.

Delete will not work if profile is a match.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | [**AccountId**](AccountId.md) |  | [required] |

### Return type

[**models::LimitedActionResult**](LimitedActionResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_matches

> models::MatchesPage get_matches()
Get matches

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::MatchesPage**](MatchesPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_message_number_of_latest_viewed_message

> models::MessageNumber get_message_number_of_latest_viewed_message(account_id)
Get message number of the most recent message that the recipient has viewed.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | [**AccountId**](AccountId.md) |  | [required] |

### Return type

[**models::MessageNumber**](MessageNumber.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_pending_messages

> std::path::PathBuf get_pending_messages()
Get list of pending messages.

The returned bytes is list of objects with following data: - UTF-8 text length encoded as 16 bit little endian number. - UTF-8 text which is PendingMessage JSON. - Binary message data length as 16 bit little endian number. - Binary message data

### Parameters

This endpoint does not need any parameter.

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_public_key

> models::GetPublicKey get_public_key(aid, version)
Get current public key of some account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**aid** | **uuid::Uuid** |  | [required] |
**version** | **i64** |  | [required] |

### Return type

[**models::GetPublicKey**](GetPublicKey.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_received_blocks

> models::ReceivedBlocksPage get_received_blocks()
Get list of received blocks

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ReceivedBlocksPage**](ReceivedBlocksPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_sent_blocks

> models::SentBlocksPage get_sent_blocks()
Get list of sent blocks

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::SentBlocksPage**](SentBlocksPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_sent_likes

> models::SentLikesPage get_sent_likes()
Get sent likes.

Profile will not be returned if:  - Profile is hidden (not public) - Profile is blocked - Profile is a match

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::SentLikesPage**](SentLikesPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_sent_message_ids

> models::SentMessageIdList get_sent_message_ids()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::SentMessageIdList**](SentMessageIdList.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_add_receiver_acknowledgement

> post_add_receiver_acknowledgement(pending_message_acknowledgement_list)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**pending_message_acknowledgement_list** | [**PendingMessageAcknowledgementList**](PendingMessageAcknowledgementList.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_add_sender_acknowledgement

> post_add_sender_acknowledgement(sent_message_id_list)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sent_message_id_list** | [**SentMessageIdList**](SentMessageIdList.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_block_profile

> post_block_profile(account_id)
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


## post_get_new_received_likes_count

> models::NewReceivedLikesCountResult post_get_new_received_likes_count()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::NewReceivedLikesCountResult**](NewReceivedLikesCountResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_get_next_received_likes_page

> models::ReceivedLikesPage post_get_next_received_likes_page(received_likes_iterator_session_id)
Update received likes iterator and get next page

of received likes. If the page is empty there is no more received likes available.  Profile will not be returned if: - Profile is blocked - Profile is a match

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**received_likes_iterator_session_id** | [**ReceivedLikesIteratorSessionId**](ReceivedLikesIteratorSessionId.md) |  | [required] |

### Return type

[**models::ReceivedLikesPage**](ReceivedLikesPage.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_get_pending_notification

> models::PendingNotificationWithData post_get_pending_notification(pending_notification_token)
Get pending notification and reset pending notification.

Requesting this route is always valid to avoid figuring out device token values more easily.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**pending_notification_token** | [**PendingNotificationToken**](PendingNotificationToken.md) |  | [required] |

### Return type

[**models::PendingNotificationWithData**](PendingNotificationWithData.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_message_number_of_latest_viewed_message

> post_message_number_of_latest_viewed_message(update_message_view_status)
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


## post_public_key

> models::PublicKeyId post_public_key(set_public_key)
Replace current public key with a new public key.

Returns public key ID number which server increments. This must be called only when needed as this route will fail every time if current public key ID number is i64::MAX.  Only version 1 public keys are currently supported.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**set_public_key** | [**SetPublicKey**](SetPublicKey.md) |  | [required] |

### Return type

[**models::PublicKeyId**](PublicKeyId.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_reset_received_likes_paging

> models::ResetReceivedLikesIteratorResult post_reset_received_likes_paging()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ResetReceivedLikesIteratorResult**](ResetReceivedLikesIteratorResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_send_like

> models::LimitedActionResult post_send_like(account_id)
Send a like to some account. If both will like each other, then

the accounts will be a match.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_id** | [**AccountId**](AccountId.md) |  | [required] |

### Return type

[**models::LimitedActionResult**](LimitedActionResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_send_message

> models::SendMessageResult post_send_message(receiver, receiver_public_key_id, receiver_public_key_version, client_id, client_local_id, body)
Send message to a match.

Max pending message count is 50. Max message size is u16::MAX.  The sender message ID must be value which server expects.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**receiver** | **uuid::Uuid** | Receiver of the message. | [required] |
**receiver_public_key_id** | **i64** | Message receiver's public key ID for check to prevent sending message encrypted with outdated public key. | [required] |
**receiver_public_key_version** | **i64** |  | [required] |
**client_id** | **i64** |  | [required] |
**client_local_id** | **i64** |  | [required] |
**body** | **std::path::PathBuf** |  | [required] |

### Return type

[**models::SendMessageResult**](SendMessageResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/octet-stream
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_set_device_token

> models::PendingNotificationToken post_set_device_token(fcm_device_token)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**fcm_device_token** | [**FcmDeviceToken**](FcmDeviceToken.md) |  | [required] |

### Return type

[**models::PendingNotificationToken**](PendingNotificationToken.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_unblock_profile

> post_unblock_profile(account_id)
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

