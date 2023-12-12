use std::fmt::Debug;

pub struct Allowed;
pub struct NotAllowed;

pub trait AllowedStatus {
    const LOGGING_ALLOWED: bool;
}
impl AllowedStatus for Allowed {
    const LOGGING_ALLOWED: bool = true;
}
impl AllowedStatus for NotAllowed {
    const LOGGING_ALLOWED: bool = false;
}

/// Control logging when server debug mode is disabled.
pub trait IsLoggingAllowed {
    type Value: AllowedStatus;
    const LOGGING_ALLOWED: bool = Self::Value::LOGGING_ALLOWED;

    fn fmt_loggable(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("-")
    }
}

macro_rules! disable_logging {
    ($($name:ty,)* ) => {
        $(
            impl IsLoggingAllowed for $name {
                type Value = NotAllowed;
            }
            impl IsLoggingAllowed for &$name {
                type Value = NotAllowed;
            }
        )*
    };
}

macro_rules! enable_logging {
    ($($name:ty,)* ) => {
        $(
            impl IsLoggingAllowed for $name {
                type Value = Allowed;

                fn fmt_loggable(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl IsLoggingAllowed for &$name {
                type Value = Allowed;

                fn fmt_loggable(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
        )*
    };
}

// enable_logging!(
//     // Account
//     AccountIdInternal,
//     AccountId,
//     Option<AccountIdDb>,
//     // Media
//     ModerationRequestIdDb,
//     ModerationRequestId, // TODO: combine with ModerationRequestIdDb
//     ContentIdDb,
//     ContentId,
//     ImageSlot,
//     ModerationId,
//     ModerationQueueNumber,
// );

// disable_logging!(
//     // Account
//     GoogleAccountId,
//     // Media
//     ModerationRequestContent,
//     PrimaryImage,
//     ContentState,
//     i64,
//     (),
// );

impl<T1: IsLoggingAllowed, T2: IsLoggingAllowed> IsLoggingAllowed for (T1, T2)
where
    Self: Debug,
{
    type Value = Allowed;

    fn fmt_loggable(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;
        self.0.fmt_loggable(f)?;
        f.write_str(", ")?;
        self.1.fmt_loggable(f)?;
        f.write_str(")")?;
        Ok(())
    }
}

impl<T1: IsLoggingAllowed + Debug, T2: IsLoggingAllowed + Debug, T3: IsLoggingAllowed + Debug>
    IsLoggingAllowed for (T1, T2, T3)
where
    Self: Debug,
{
    type Value = Allowed;
    fn fmt_loggable(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;
        self.0.fmt_loggable(f)?;
        f.write_str(", ")?;
        self.1.fmt_loggable(f)?;
        f.write_str(", ")?;
        self.2.fmt_loggable(f)?;
        f.write_str(")")?;
        Ok(())
    }
}

impl<
        T1: IsLoggingAllowed + Debug,
        T2: IsLoggingAllowed + Debug,
        T3: IsLoggingAllowed + Debug,
        T4: IsLoggingAllowed + Debug,
    > IsLoggingAllowed for (T1, T2, T3, T4)
where
    Self: Debug,
{
    type Value = Allowed;
    fn fmt_loggable(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;
        self.0.fmt_loggable(f)?;
        f.write_str(", ")?;
        self.1.fmt_loggable(f)?;
        f.write_str(", ")?;
        self.2.fmt_loggable(f)?;
        f.write_str(", ")?;
        self.3.fmt_loggable(f)?;
        f.write_str(")")?;
        Ok(())
    }
}
