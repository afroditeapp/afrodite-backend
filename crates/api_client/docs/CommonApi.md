# \CommonApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_connect_websocket**](CommonApi.md#get_connect_websocket) | **GET** /common_api/connect | Connect to server using WebSocket after getting refresh and access tokens.
[**get_version**](CommonApi.md#get_version) | **GET** /common_api/version | Get backend version.



## get_connect_websocket

> get_connect_websocket()
Connect to server using WebSocket after getting refresh and access tokens.

Connect to server using WebSocket after getting refresh and access tokens. Connection is required as API access is allowed for connected clients.  Send the current refersh token as Binary. The server will send the next refresh token (Binary) and after that the new access token (Text). After that API can be used.  The access token is valid until this WebSocket is closed. Server might send events as Text which is JSON. 

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


## get_version

> crate::models::BackendVersion get_version()
Get backend version.

Get backend version.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::BackendVersion**](BackendVersion.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

