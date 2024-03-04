
# Profile filtering

Profile contains serveral attributes that can be used to filter profiles. The
filters are dynamically specified on the server using TOML file. That file
is then converted to a JSON object which client uses to implement
displaying and editing of the attributes.


## TOML file format

```toml

attribute_order = "OrderNumber"

[[attribute]]
key = "city"
name = "City"
mode = "SelectSingleFilterSingle"
editable = true # Optional
visible = true # Optional
required = false # Optional
icon = "material:location_city"
id = 0
order = 0
value_order = "AlphabeticalValue"
values = [
    {
        key = "helsinki",
        value = "Helsinki",
        id = 0, # Optional
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
            key = "kallio",
            value = "Kallio",
            id = 0, # Optional
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
            { key = "city", value = "Stad" },
            { key = "helsinki", value = "Helsingfors" },
            { key = "kallio", value = "Berghäll" },
        ]
    }
]

[[attribute]]
key = "favorite_color"
name = "Favorite Color"
mode = "SelectSingleFilterMultiple"
icon = "material:color_lens"
id = 1
order = 1
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
- `name` - English name for the attribute
- `mode` - mode of the attribute. Possible values are
    - `SelectSingleFilterSingle` - single value select filter. Top and sub
        level values are possible to set. Max value count for top and sub
        level are u16::MAX.
    - `SelectSingleFilterMultiple` - multiple values in select filter.
        Only top level values are possible to set. Max value count is 7.
        (Internal representation is 8 bit bitflag and zero bit is
        reserved for filtering purposes)
    - `SelectMultipleFilterMultiple` - same as `SelectSingleFilterMultiple`
        but selecting multiple bitflags are possible.
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
    - `value` - English translation for the value
    - `id` - Unique ID number for the value for the value. Beharior is
        is different depending on the `type` attribute.
        - Behavior for `SelectSingleFilterSingle` - Default value starts
            from 0 and default value for next list item is the previous + 1.
            Max value is u16::MAX.
        - Behavior for `SelectSingleFilterMultiple` - Default value starts
            from 0x2 and default value for next list item is the
            previous << 1. Max value is 0x80 so 7 values are possible
            to define.
        The field is optional.
    - `icon` - Icon to be used for the attribute.
        This field has the same format as the attribute `icon` field.
        (default: null)
    - `order_number` - Display order number for the value.
        0 is the first value.
        Default value starts from 0 and default value for next
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
        - `value` - translation for the key

If attirbute `type` is `SelectSingleFilterSingle` the following fields is
possible to set:

- `group_values` - list of possible sub values for top level values.
    Contains strings or objects with fields
    - `key` - unique identifier for the top level value.
    - `values` - list of possible sub level values for the top level value.
        Contains objects with fields
        - `key` - unique identifier for the value
        - `value` - English translation for the value
        - `id` - Unique ID number for the value for the value. Beharior is
            is different depending on the `type` attribute.
            - Behavior for `SelectSingleFilterSingle` - Default value starts
                from 0 and default value for next list item is the previous + 1.
                Max value is u16::MAX.
            - Behavior for `SelectSingleFilterMultiple` - Not possible to set.
            The field is optional.
        - `icon` - Icon to be used for the attribute.
            This field has the same format as the attribute `icon` field.
            (default: null)
        - `order_number` - Display order number for the value.
            0 is the first value.
            Default value starts from 0 and default value for next
            list item is the previous + 1.
            The field is optional.
        - `editable` - boolean value to state if the value is visible
            in client's profile editing view
            (default: true)
        - `visible` - boolean value to state if the value is visible
            in client's profile view
            (default: true)
