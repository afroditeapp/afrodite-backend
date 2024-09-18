# LoginResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account** | Option<[**models::AuthPair**](AuthPair.md)> |  | [optional]
**aid** | Option<[**models::AccountId**](AccountId.md)> |  | [optional]
**email** | Option<**String**> |  | [optional]
**error_unsupported_client** | Option<**bool**> |  | [optional][default to false]
**latest_public_keys** | Option<[**Vec<models::PublicKeyIdAndVersion>**](PublicKeyIdAndVersion.md)> | Info about latest public keys. Client can use this value to ask if user wants to copy existing private and public key from other device. If empty, public key is not set or the client is unsupported. | [optional][default to []]
**media** | Option<[**models::AuthPair**](AuthPair.md)> |  | [optional]
**profile** | Option<[**models::AuthPair**](AuthPair.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


