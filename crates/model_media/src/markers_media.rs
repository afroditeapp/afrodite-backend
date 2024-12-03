use super::*;

enable_logging!(
    // Media
    ContentIdDb,
);

disable_logging!(
    // Media
    ProfileContent,
    SetProfileContent,
    SetProfileContentInternal,
);
