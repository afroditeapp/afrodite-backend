struct InternalVerificationStatusFlags {}

impl InternalVerificationStatusFlags {
    const PROFILE_CONTENT_FACE_VERIFIED_ANY: i16 = 0x1;
    const PROFILE_CONTENT_FACE_VERIFIED_ALL: i16 = 0x2;
    const SECURITY_CONTENT_VERIFIED: i16 = 0x4;
    const PROFILE_AGE_RANGE_VERIFIED: i16 = 0x8;
    const PROFILE_NAME_VERIFIED: i16 = 0x10;
}

bitflags::bitflags! {
    /// Profile-level verification status flags calculated from media content.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MediaVerificationStatusFlags: i16 {
        /// At least one current profile picture has effective face verified value true.
        const PROFILE_CONTENT_FACE_VERIFIED_ANY = InternalVerificationStatusFlags::PROFILE_CONTENT_FACE_VERIFIED_ANY;
        /// All current profile pictures have effective face verified value true.
        /// For empty profile picture list this bit must be unset.
        const PROFILE_CONTENT_FACE_VERIFIED_ALL = InternalVerificationStatusFlags::PROFILE_CONTENT_FACE_VERIFIED_ALL;
        /// Current security content has effective security verified value true.
        const SECURITY_CONTENT_VERIFIED = InternalVerificationStatusFlags::SECURITY_CONTENT_VERIFIED;
    }
}

bitflags::bitflags! {
    /// Profile-only verification status flags.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ProfileVerificationStatusFlags: i16 {
        /// Profile age range is verified.
        const PROFILE_AGE_RANGE_VERIFIED = InternalVerificationStatusFlags::PROFILE_AGE_RANGE_VERIFIED;
        /// Profile name is verified.
        const PROFILE_NAME_VERIFIED = InternalVerificationStatusFlags::PROFILE_NAME_VERIFIED;
    }
}

impl ProfileVerificationStatusFlags {
    pub fn from_profile_verification_values(
        profile_age_range_verified: Option<bool>,
        profile_name_verified: Option<bool>,
    ) -> Self {
        let mut bits = 0;
        if profile_age_range_verified == Some(true) {
            bits |= Self::PROFILE_AGE_RANGE_VERIFIED.bits();
        }
        if profile_name_verified == Some(true) {
            bits |= Self::PROFILE_NAME_VERIFIED.bits();
        }

        Self::from_bits_retain(bits)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AllVerificationStatusFlags {
    flags: i16,
}

impl AllVerificationStatusFlags {
    pub fn new(
        profile: ProfileVerificationStatusFlags,
        media: MediaVerificationStatusFlags,
    ) -> Self {
        Self {
            flags: profile.bits() | media.bits(),
        }
    }

    pub fn has_required_mask(self, required_mask: VerificationStatusFilterFlags) -> bool {
        (self.flags & required_mask.bits()) == required_mask.bits()
    }
}

bitflags::bitflags! {
    /// Profile-level verification status flags used for profile filtering.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VerificationStatusFilterFlags: i16 {
        /// At least one current profile picture has effective face verified value true.
        const PROFILE_CONTENT_FACE_VERIFIED_ANY = InternalVerificationStatusFlags::PROFILE_CONTENT_FACE_VERIFIED_ANY;
        /// All current profile pictures have effective face verified value true.
        /// For empty profile picture list this bit must be unset.
        const PROFILE_CONTENT_FACE_VERIFIED_ALL = InternalVerificationStatusFlags::PROFILE_CONTENT_FACE_VERIFIED_ALL;
        /// Current security content has effective security verified value true.
        const SECURITY_CONTENT_VERIFIED = InternalVerificationStatusFlags::SECURITY_CONTENT_VERIFIED;
        /// Profile age range is verified.
        const PROFILE_AGE_RANGE_VERIFIED = InternalVerificationStatusFlags::PROFILE_AGE_RANGE_VERIFIED;
        /// Profile name is verified.
        const PROFILE_NAME_VERIFIED = InternalVerificationStatusFlags::PROFILE_NAME_VERIFIED;
    }
}

impl TryFrom<i16> for VerificationStatusFilterFlags {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::from_bits(value).ok_or_else(|| "Unknown bitflag".to_string())
    }
}
