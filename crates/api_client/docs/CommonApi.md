# \CommonApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_connect_websocket**](CommonApi.md#get_connect_websocket) | **GET** /6qQZ2jQO5exMKFI2jCzGAdMysxE | Connect to server using WebSocket after getting refresh and access tokens.
[**get_version**](CommonApi.md#get_version) | **GET** /0E_N3KS0gI2PPBujveMhHLv292Y | Get backend version.



## get_connect_websocket

> get_connect_websocket()
Connect to server using WebSocket after getting refresh and access tokens.

Connection is required as API access is allowed for connected clients.  Protocol: 1. Client sends version information as Binary message, where - u8: Client WebSocket protocol version (currently 0). - u8: Client type number. (0 = Android, 1 = iOS, 2 = Web, 255 = Test mode bot) - u16: Client Major version. - u16: Client Minor version. - u16: Client Patch version.  The u16 values are in little endian byte order. 2. Client sends current refresh token as Binary message. 3. If server supports the client, the server sends next refresh token as Binary message. If server does not support the client, the server sends Text message and closes the connection. 4. Server sends new access token as Binary message. The client must convert the token to base64url encoding without padding. (At this point API can be used.) 5. Client sends list of current data sync versions as Binary message, where items are [u8; 2] and the first u8 of an item is the data type number and the second u8 of an item is the sync version number for that data. If client does not have any version of the data, the client should send 255 as the version number.  Available data types: - 0: Account 6. Server starts to send JSON events as Text messages and empty binary messages to test connection to the client. Client can ignore the empty binary messages. 7. If needed, the client sends empty binary messages to test connection to the server.  The new access token is valid until this WebSocket is closed or the server detects a timeout. To prevent the timeout the client must send a WebScoket ping message before 6 minutes elapses from connection establishment or previous ping message.  `Sec-WebSocket-Protocol` header must have 2 protocols/values. The first is \"0\" and that protocol is accepted. The second is access token of currently logged in account. The token is base64url encoded without padding.

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_version

> models::BackendVersion get_version()
Get backend version.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::BackendVersion**](BackendVersion.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

