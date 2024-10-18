# AttributeValue

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**editable** | Option<**bool**> |  | [optional][default to true]
**group_values** | Option<[**models::GroupValues**](GroupValues.md)> | Sub level values for this attribute value. | [optional]
**icon** | Option<**String**> |  | [optional]
**id** | **i32** | Numeric unique identifier for the attribute value. Note that the value must only be unique within a group of values, so value in top level group A, sub level group C and sub level group B can have the same ID. | 
**key** | **String** | Unique string identifier for the attribute value. | 
**order_number** | **i32** | Order number for client to determine in what order the values should be displayed. | 
**value** | **String** | English text for the attribute value. | 
**visible** | Option<**bool**> |  | [optional][default to true]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


