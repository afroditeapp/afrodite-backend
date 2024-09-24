# Attribute

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**editable** | Option<**bool**> | Client should show this attribute when editing a profile. | [optional][default to true]
**icon** | Option<**String**> | Icon for the attribute. | [optional]
**id** | **i32** | Numeric unique identifier for the attribute. | 
**key** | **String** | String unique identifier for the attribute. | 
**mode** | [**models::AttributeMode**](AttributeMode.md) |  | 
**name** | **String** | English text for the attribute. | 
**order_number** | **i32** | Attribute order number. | 
**required** | Option<**bool**> | Client should ask this attribute when doing account initial setup. | [optional][default to false]
**translations** | Option<[**Vec<models::Language>**](Language.md)> | Translations for attribute name and attribute values. | [optional][default to []]
**value_order** | [**models::AttributeValueOrderMode**](AttributeValueOrderMode.md) |  | 
**values** | [**Vec<models::AttributeValue>**](AttributeValue.md) | Top level values for the attribute.  Values are sorted by AttributeValue ID. Indexing with it is not possible as ID might be a bitflag value. | 
**visible** | Option<**bool**> | Client should show this attribute when viewing a profile. | [optional][default to true]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


