bitflags::bitflags! {
    /// Profile-level verification status flags calculated from media content.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MediaVerificationStatusFlags: i16 {
        /// At least one current profile picture has effective face verified value true.
        const PROFILE_CONTENT_FACE_VERIFIED_ANY = 0x1;
        /// All current profile pictures have effective face verified value true.
        /// For empty profile picture list this bit must be unset.
        const PROFILE_CONTENT_FACE_VERIFIED_ALL = 0x2;
        /// Current security content has effective security verified value true.
        const SECURITY_CONTENT_VERIFIED = 0x4;
    }
}

impl MediaVerificationStatusFlags {
    pub fn has_required_mask(self, required_mask: VerificationStatusFilterFlags) -> bool {
        (self.bits() & required_mask.bits()) == required_mask.bits()
    }
}

bitflags::bitflags! {
    /// Profile-level verification status flags used for profile filtering.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VerificationStatusFilterFlags: i16 {
        /// At least one current profile picture has effective face verified value true.
        const PROFILE_CONTENT_FACE_VERIFIED_ANY = MediaVerificationStatusFlags::PROFILE_CONTENT_FACE_VERIFIED_ANY.bits();
        /// All current profile pictures have effective face verified value true.
        /// For empty profile picture list this bit must be unset.
        const PROFILE_CONTENT_FACE_VERIFIED_ALL = MediaVerificationStatusFlags::PROFILE_CONTENT_FACE_VERIFIED_ALL.bits();
        /// Current security content has effective security verified value true.
        const SECURITY_CONTENT_VERIFIED = MediaVerificationStatusFlags::SECURITY_CONTENT_VERIFIED.bits();
    }
}

impl TryFrom<i16> for VerificationStatusFilterFlags {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::from_bits(value).ok_or_else(|| "Unknown bitflag".to_string())
    }
}
