use model_server_data::SearchGroupFlags;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// My gender and what gender I'm searching for.
///
/// Fileds should be read "I'm x and I'm searching for y".
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq, Default)]
pub struct SearchGroups {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")] // Skips false
    #[schema(default = false)]
    pub man_for_woman: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub man_for_man: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub man_for_non_binary: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub woman_for_man: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub woman_for_woman: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub woman_for_non_binary: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub non_binary_for_man: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub non_binary_for_woman: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub non_binary_for_non_binary: bool,
}

impl SearchGroups {
    fn to_validated_man(self) -> Option<ValidatedSearchGroups> {
        if self.man_for_woman || self.man_for_man || self.man_for_non_binary {
            Some(ValidatedSearchGroups::ManFor {
                woman: self.man_for_woman,
                man: self.man_for_man,
                non_binary: self.man_for_non_binary,
            })
        } else {
            None
        }
    }

    fn to_validated_woman(self) -> Option<ValidatedSearchGroups> {
        if self.woman_for_man || self.woman_for_woman || self.woman_for_non_binary {
            Some(ValidatedSearchGroups::WomanFor {
                man: self.woman_for_man,
                woman: self.woman_for_woman,
                non_binary: self.woman_for_non_binary,
            })
        } else {
            None
        }
    }

    fn to_validated_non_binary(self) -> Option<ValidatedSearchGroups> {
        if self.non_binary_for_man || self.non_binary_for_woman || self.non_binary_for_non_binary {
            Some(ValidatedSearchGroups::NonBinaryFor {
                man: self.non_binary_for_man,
                woman: self.non_binary_for_woman,
                non_binary: self.non_binary_for_non_binary,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidatedSearchGroups {
    ManFor {
        woman: bool,
        man: bool,
        non_binary: bool,
    },
    WomanFor {
        man: bool,
        woman: bool,
        non_binary: bool,
    },
    NonBinaryFor {
        man: bool,
        woman: bool,
        non_binary: bool,
    },
}

impl TryFrom<SearchGroups> for ValidatedSearchGroups {
    type Error = &'static str;

    fn try_from(value: SearchGroups) -> Result<Self, Self::Error> {
        match (
            value.to_validated_man(),
            value.to_validated_woman(),
            value.to_validated_non_binary(),
        ) {
            (Some(v), None, None) => Ok(v),
            (None, Some(v), None) => Ok(v),
            (None, None, Some(v)) => Ok(v),
            (None, None, None) => Err("Gender not set"),
            _ => Err("Unambiguous gender"),
        }
    }
}

impl From<ValidatedSearchGroups> for SearchGroupFlags {
    fn from(value: ValidatedSearchGroups) -> Self {
        let mut flags: SearchGroupFlags = Self::empty();
        match value {
            ValidatedSearchGroups::ManFor {
                woman,
                man,
                non_binary,
            } => {
                if woman {
                    flags |= Self::MAN_FOR_WOMAN;
                }
                if man {
                    flags |= Self::MAN_FOR_MAN;
                }
                if non_binary {
                    flags |= Self::MAN_FOR_NON_BINARY;
                }
            }
            ValidatedSearchGroups::WomanFor {
                man,
                woman,
                non_binary,
            } => {
                if man {
                    flags |= Self::WOMAN_FOR_MAN;
                }
                if woman {
                    flags |= Self::WOMAN_FOR_WOMAN;
                }
                if non_binary {
                    flags |= Self::WOMAN_FOR_NON_BINARY;
                }
            }
            ValidatedSearchGroups::NonBinaryFor {
                man,
                woman,
                non_binary,
            } => {
                if man {
                    flags |= Self::NON_BINARY_FOR_MAN;
                }
                if woman {
                    flags |= Self::NON_BINARY_FOR_WOMAN;
                }
                if non_binary {
                    flags |= Self::NON_BINARY_FOR_NON_BINARY;
                }
            }
        }
        flags
    }
}

impl From<SearchGroupFlags> for SearchGroups {
    fn from(v: SearchGroupFlags) -> Self {
        Self {
            man_for_woman: v.contains(SearchGroupFlags::MAN_FOR_WOMAN),
            man_for_man: v.contains(SearchGroupFlags::MAN_FOR_MAN),
            man_for_non_binary: v.contains(SearchGroupFlags::MAN_FOR_NON_BINARY),
            woman_for_man: v.contains(SearchGroupFlags::WOMAN_FOR_MAN),
            woman_for_woman: v.contains(SearchGroupFlags::WOMAN_FOR_WOMAN),
            woman_for_non_binary: v.contains(SearchGroupFlags::WOMAN_FOR_NON_BINARY),
            non_binary_for_man: v.contains(SearchGroupFlags::NON_BINARY_FOR_MAN),
            non_binary_for_woman: v.contains(SearchGroupFlags::NON_BINARY_FOR_WOMAN),
            non_binary_for_non_binary: v.contains(SearchGroupFlags::NON_BINARY_FOR_NON_BINARY),
        }
    }
}
