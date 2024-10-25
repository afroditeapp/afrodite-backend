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

bitflags::bitflags! {
    /// Same as SearchGroups but as bitflags. The biflags are used in database.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct SearchGroupFlags: u16 {
        const MAN_FOR_WOMAN = 0x1;
        const MAN_FOR_MAN = 0x2;
        const MAN_FOR_NON_BINARY = 0x4;
        const WOMAN_FOR_MAN = 0x8;
        const WOMAN_FOR_WOMAN = 0x10;
        const WOMAN_FOR_NON_BINARY = 0x20;
        const NON_BINARY_FOR_MAN = 0x40;
        const NON_BINARY_FOR_WOMAN = 0x80;
        const NON_BINARY_FOR_NON_BINARY = 0x100;
    }
}

impl SearchGroupFlags {
    pub fn to_filter(&self) -> SearchGroupFlagsFilter {
        SearchGroupFlagsFilter::new(*self)
    }

    pub fn is_man(self) -> bool {
        let groups: SearchGroups = self.into();
        groups.to_validated_man().is_some()
    }

    pub fn is_woman(self) -> bool {
        let groups: SearchGroups = self.into();
        groups.to_validated_woman().is_some()
    }

    pub fn is_non_binary(self) -> bool {
        let groups: SearchGroups = self.into();
        groups.to_validated_non_binary().is_some()
    }
}

impl TryFrom<i64> for SearchGroupFlags {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = TryInto::<u16>::try_into(value).map_err(|e| e.to_string())?;
        Self::from_bits(value).ok_or_else(|| "Unknown bitflag".to_string())
    }
}

impl From<SearchGroupFlags> for i64 {
    fn from(value: SearchGroupFlags) -> Self {
        value.bits() as i64
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

/// Filter which finds matches with other SearchGroupFlags.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SearchGroupFlagsFilter {
    filter: SearchGroupFlags,
}

impl SearchGroupFlagsFilter {
    fn new(flags: SearchGroupFlags) -> Self {
        let mut filter = SearchGroupFlags::empty();

        // Man
        if flags.contains(SearchGroupFlags::MAN_FOR_WOMAN) {
            filter |= SearchGroupFlags::WOMAN_FOR_MAN;
        }
        if flags.contains(SearchGroupFlags::MAN_FOR_MAN) {
            filter |= SearchGroupFlags::MAN_FOR_MAN;
        }
        if flags.contains(SearchGroupFlags::MAN_FOR_NON_BINARY) {
            filter |= SearchGroupFlags::NON_BINARY_FOR_MAN;
        }
        // Woman
        if flags.contains(SearchGroupFlags::WOMAN_FOR_MAN) {
            filter |= SearchGroupFlags::MAN_FOR_WOMAN;
        }
        if flags.contains(SearchGroupFlags::WOMAN_FOR_WOMAN) {
            filter |= SearchGroupFlags::WOMAN_FOR_WOMAN;
        }
        if flags.contains(SearchGroupFlags::WOMAN_FOR_NON_BINARY) {
            filter |= SearchGroupFlags::NON_BINARY_FOR_WOMAN;
        }
        // Non-binary
        if flags.contains(SearchGroupFlags::NON_BINARY_FOR_MAN) {
            filter |= SearchGroupFlags::MAN_FOR_NON_BINARY;
        }
        if flags.contains(SearchGroupFlags::NON_BINARY_FOR_WOMAN) {
            filter |= SearchGroupFlags::WOMAN_FOR_NON_BINARY;
        }
        if flags.contains(SearchGroupFlags::NON_BINARY_FOR_NON_BINARY) {
            filter |= SearchGroupFlags::NON_BINARY_FOR_NON_BINARY;
        }

        Self { filter }
    }

    pub(crate) fn is_match(&self, flags: SearchGroupFlags) -> bool {
        self.filter.intersects(flags)
    }
}
