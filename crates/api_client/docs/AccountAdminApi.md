# \AccountAdminApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_news_item**](AccountAdminApi.md#delete_news_item) | **DELETE** /BUFRdjIQCtPBjy00uEOHIA9X8CI/{nid} | 
[**delete_news_translation**](AccountAdminApi.md#delete_news_translation) | **DELETE** /BUFRdjIQCtPBjy00uEOHIA9X8CI/{nid}/{locale} | 
[**post_create_news_item**](AccountAdminApi.md#post_create_news_item) | **POST** /XEss8YDw9lPgwKoH6K9THZIF_N4 | 
[**post_set_news_publicity**](AccountAdminApi.md#post_set_news_publicity) | **DELETE** /VhK61QQYLFov-eCH2YS2i5M2jdk/{nid} | 
[**post_update_news_translation**](AccountAdminApi.md#post_update_news_translation) | **POST** /4pD-Q4FhZGTNkUGYExHmZN6TxjU/{nid}/{locale} | 



## delete_news_item

> delete_news_item(nid)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**nid** | **i64** |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_news_translation

> delete_news_translation(nid, locale)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**nid** | **i64** |  | [required] |
**locale** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_create_news_item

> models::NewsId post_create_news_item()


### Parameters

This endpoint does not need any parameter.

### Return type

[**models::NewsId**](NewsId.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_set_news_publicity

> post_set_news_publicity(nid, boolean_setting)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**nid** | **i64** |  | [required] |
**boolean_setting** | [**BooleanSetting**](BooleanSetting.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_update_news_translation

> post_update_news_translation(nid, locale, update_news_translation)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**nid** | **i64** |  | [required] |
**locale** | **String** |  | [required] |
**update_news_translation** | [**UpdateNewsTranslation**](UpdateNewsTranslation.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

