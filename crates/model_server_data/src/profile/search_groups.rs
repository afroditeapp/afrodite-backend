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
        self.contains(Self::MAN_FOR_WOMAN)
            || self.contains(Self::MAN_FOR_MAN)
            || self.contains(Self::MAN_FOR_NON_BINARY)
    }

    pub fn is_woman(self) -> bool {
        self.contains(Self::WOMAN_FOR_WOMAN)
            || self.contains(Self::WOMAN_FOR_MAN)
            || self.contains(Self::WOMAN_FOR_NON_BINARY)
    }

    pub fn is_non_binary(self) -> bool {
        self.contains(Self::NON_BINARY_FOR_WOMAN)
            || self.contains(Self::NON_BINARY_FOR_MAN)
            || self.contains(Self::NON_BINARY_FOR_NON_BINARY)
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

    pub fn is_searching_men(&self) -> bool {
        self.filter.is_man()
    }

    pub fn is_searching_women(&self) -> bool {
        self.filter.is_woman()
    }

    pub fn is_searching_non_binaries(&self) -> bool {
        self.filter.is_non_binary()
    }
}
