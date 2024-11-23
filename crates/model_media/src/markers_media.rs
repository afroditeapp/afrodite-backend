use super::*;

enable_logging!(
    // Media
    ModerationRequestIdDb,
    ModerationRequestId, // TODO: combine with ModerationRequestIdDb
    ContentIdDb,
    ContentSlot,
    ModerationId,
);

disable_logging!(
    // Media
    ModerationRequestContent,
    ProfileContent,
    SetProfileContent,
    SetProfileContentInternal,
    ContentState,
);
