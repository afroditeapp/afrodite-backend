
# Profile filtering

Profile contains serveral attributes that can be used to filter profiles. The
filters are dynamically specified on the server using TOML file. That file
is then converted to a JSON object which client uses to implement
displaying and editing of the attributes.

All ID and order number values start from 1 for consistency so 0 can be used
as null value for creating filter integer for group values. For example u32
with most significant bits have u16 attribute value ID and least significant
bits have u16 zero value for indicating that match with all group values.
The attribute value ID is the significant value so that values have correct
order.

## TOML file format

```toml

attribute_order = "OrderNumber"

[[attribute]]
key = "city"
name = "City"
mode = "TwoLevel"
editable = true # Optional
visible = true # Optional
required = false # Optional
icon = "material:location_city"
id = 1
order_number = 1
value_order = "AlphabeticalValue"
values = [
    {
        key = "helsinki", # Optional
        name = "Helsinki",
        id = 1, # Optional
        editable = true, # Optional
        visible = true, # Optional
        icon = null # Optional
    },
    # Or simply just
    # "Helsinki",
    "Espoo",
]
group_values = [ # Optional
    { key = "helsinki", values = [
        {
            key = "kallio", # Optional
            name = "Kallio",
            id = 1, # Optional
            editable = true, # Optional
            visible = true, # Optional
            icon = null # Optional
        },
        # Or simply just
        # "Kallio",
    ] },
]
translations = [ # Optonal
    {
        lang = "se",
        values = [
            { key = "city", name = "Stad" },
            { key = "helsinki", name = "Helsingfors" },
            { key = "kallio", name = "Berghäll" },
        ]
    }
]

[[attribute]]
key = "favorite_color"
name = "Favorite Color"
mode = "Bitflag"
icon = "material:color_lens"
id = 2
order_number = 2
values = [
    "Red",   # id = 0x2
    "Green", # id = 0x4
    "Blue",  # id = 0x8
]


```
### Top level fields

- `attribute_order` - Display order mode for attributes.
    Possible values are
    - `OrderNumber` - use the order numbers to sort the attributes.

### Attribute section fields

#### Required fields
- `key` - unique identifier of the attribute
- `name` - Default name for the attribute
- `mode` - mode of the attribute. Possible values are
    - `Bitflag` - u16 bitflags.
        Only top level values are possible to set.
        Max value count: 16.
    - `OneLevel` - u16 values.
        Only top level values are possible to set.
        Max value count: u16::MAX.
    - `TwoLevel` - u32 values.
        Top and sub level values are possible to set.
        Max value count for top and sub level are u16::MAX.
- `max_selected` - Optional max value count for selected attribute values.
    Default and min value is 1.
- `max_filters` - Optional max value count for selected filter values.
    Default and min value is 1.
- `order_number` - Unique order number for the attribute.
        0 is the first attribute.
- `value_order` - Display order mode for the attribute values.
    Possible values are
    - `AlphabeticalKey` - ignore order numbers and sort the attribute values
        alphabetically using attribute value key.
    - `AlphabeticalValue` - ignore order numbers and sort the attribute
      values alphabetically using displayed attribute value/translation.
    - `OrderNumber` - use the order numbers to sort the attribute values.
- `icon` - icon to be used for the attribute. The format is
        `src:icon_identifier`. The `src` value `material` states
        that the `icon_identifier` value is from the material icon set.
- `id` - unique numeric ID for the attribute. This is used in database
        level.
- `values` - list of possible top level values for the attribute.
    Contains strings or objects with fields
    - `key` - unique identifier for the value
    - `name` - Default name for the value
    - `id` - Unique ID number for the value for the value. Beharior is
        is different depending on the `type` attribute.
        - Behavior for bitflag attributes - Default value starts
            from 0x1 and default value for next list item is the
            previous << 1. Last value is 0x8000 so 16 values are possible
            to define.
        - Behavior for other attributes - Default value starts
            from 1 and default value for next list item is the previous + 1.
            Max value is u16::MAX.
        The field is optional.
    - `icon` - Icon to be used for the attribute.
        This field has the same format as the attribute `icon` field.
        (default: null)
    - `order_number` - Display order number for the value.
        1 is the first value.
        Default value starts from 1 and default value for next
        list item is the previous + 1.
        The field is optional.
    - `editable` - boolean value to state if the value is visible
            in client's profile editing view
            (default: true)
    - `visible` - boolean value to state if the value is visible
        in client's profile view
        (default: true)

#### Optional fields

- `editable` - boolean value to state if the attribute is
                visible in client's profile editing view
               (default: true)
- `visible` - boolean value to state if the attribute is visible
                in client's profile view
                (default: true)
- `required` - boolean value to state if the attribute must be set
                when client sets up the account
                (default: false)
- `translations` - list of language objects which has fields
    - `lang` - language code
    - `values` - list of translation objects with fields
        - `key` - key of the attribute or value
        - `name` - translation for the key

If attirbute `type` is two level attribute the following fields is
possible to set:

- `group_values` - list of possible sub values for top level values.
    Contains strings or objects with fields
    - `key` - unique identifier for the top level value.
    - `values` - list of possible sub level values for the top level value.
        Contains objects with fields
        - `key` - unique identifier for the value
        - `name` - Default name for the value
        - `id` - Unique ID number for the value for the value.
            Default value starts from 1 and default value for
            next list item is the previous + 1.
            Max value is u16::MAX.
            The field is optional.
        - `icon` - Icon to be used for the attribute.
            This field has the same format as the attribute `icon` field.
            (default: null)
        - `order_number` - Display order number for the value.
            1 is the first value.
            Default value starts from 1 and default value for next
            list item is the previous + 1.
            The field is optional.
        - `editable` - boolean value to state if the value is visible
            in client's profile editing view
            (default: true)
        - `visible` - boolean value to state if the value is visible
            in client's profile view
            (default: true)
