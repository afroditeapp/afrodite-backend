# \CommonAdminApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_backend_config**](CommonAdminApi.md#get_backend_config) | **GET** /common_api/backend_config | Get dynamic backend config.
[**get_latest_build_info**](CommonAdminApi.md#get_latest_build_info) | **GET** /common_api/get_latest_build_info | Get latest software build information available for update from manager
[**get_perf_data**](CommonAdminApi.md#get_perf_data) | **GET** /common_api/perf_data | Get performance data
[**get_software_info**](CommonAdminApi.md#get_software_info) | **GET** /common_api/software_info | Get software version information from manager instance.
[**get_system_info**](CommonAdminApi.md#get_system_info) | **GET** /common_api/system_info | Get system information from manager instance.
[**post_backend_config**](CommonAdminApi.md#post_backend_config) | **POST** /common_api/backend_config | Save dynamic backend config.
[**post_request_build_software**](CommonAdminApi.md#post_request_build_software) | **POST** /common_api/request_build_software | Request building new software from manager instance.
[**post_request_restart_or_reset_backend**](CommonAdminApi.md#post_request_restart_or_reset_backend) | **POST** /common_api/request_restart_or_reset_backend | Request restarting or reseting backend through app-manager instance.
[**post_request_update_software**](CommonAdminApi.md#post_request_update_software) | **POST** /common_api/request_update_software | Request updating new software from manager instance.



## get_backend_config

> models::BackendConfig get_backend_config()
Get dynamic backend config.

Get dynamic backend config.  # Capabilities Requires admin_server_maintenance_view_backend_settings.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::BackendConfig**](BackendConfig.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_latest_build_info

> models::BuildInfo get_latest_build_info(software_options)
Get latest software build information available for update from manager

Get latest software build information available for update from manager instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**software_options** | [**SoftwareOptions**](.md) |  | [required] |

### Return type

[**models::BuildInfo**](BuildInfo.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_perf_data

> models::PerfHistoryQueryResult get_perf_data(start_time, end_time)
Get performance data

Get performance data  # Capabilities Requires admin_server_maintenance_view_info.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**start_time** | Option<[**models::UnixTime**](.md)> | Start time for query results. |  |
**end_time** | Option<[**models::UnixTime**](.md)> | End time for query results. |  |

### Return type

[**models::PerfHistoryQueryResult**](PerfHistoryQueryResult.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_software_info

> models::SoftwareInfo get_software_info()
Get software version information from manager instance.

Get software version information from manager instance.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::SoftwareInfo**](SoftwareInfo.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_system_info

> models::SystemInfoList get_system_info()
Get system information from manager instance.

Get system information from manager instance.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::SystemInfoList**](SystemInfoList.md)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_backend_config

> post_backend_config(backend_config)
Save dynamic backend config.

Save dynamic backend config.  # Capabilities Requires admin_server_maintenance_save_backend_settings.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backend_config** | [**BackendConfig**](BackendConfig.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_request_build_software

> post_request_build_software(software_options)
Request building new software from manager instance.

Request building new software from manager instance.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**software_options** | [**SoftwareOptions**](.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_request_restart_or_reset_backend

> post_request_restart_or_reset_backend(reset_data)
Request restarting or reseting backend through app-manager instance.

Request restarting or reseting backend through app-manager instance.  # Capabilities Requires admin_server_maintenance_restart_backend. Also requires admin_server_maintenance_reset_data if reset_data is true.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**reset_data** | **bool** |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_request_update_software

> post_request_update_software(software_options, reboot, reset_data)
Request updating new software from manager instance.

Request updating new software from manager instance.  Reboot query parameter will force reboot of the server after update. If it is off, the server will be rebooted when the usual reboot check is done.  Reset data query parameter will reset data like defined in current app-manager version. If this is true then specific capability is needed for completing this request.  # Capablities Requires admin_server_maintenance_update_software. Also requires admin_server_maintenance_reset_data if reset_data is true.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**software_options** | [**SoftwareOptions**](.md) |  | [required] |
**reboot** | **bool** |  | [required] |
**reset_data** | **bool** |  | [required] |

### Return type

 (empty response body)

### Authorization

[access_token](../README.md#access_token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

