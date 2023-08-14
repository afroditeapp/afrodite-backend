

use std::fmt::Debug;

use super::*;


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

#[derive(Default)]
pub struct DbgPrinter<'a> {
    value1: Option<&'a dyn std::fmt::Debug>,
    value2: Option<&'a dyn std::fmt::Debug>,
    value3: Option<&'a dyn std::fmt::Debug>,
    value4: Option<&'a dyn std::fmt::Debug>,
}

impl <'a> Debug for DbgPrinter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg_tuple = f.debug_tuple("");
        if let Some(value1) = &self.value1 {
            dbg_tuple.field(value1);
        }
        if let Some(value2) = &self.value2 {
            dbg_tuple.field(value2);
        }
        if let Some(value3) = &self.value3 {
            dbg_tuple.field(value3);
        }
        if let Some(value4) = &self.value4 {
            dbg_tuple.field(value4);
        }
        dbg_tuple.finish()
    }
}


/// Control logging when server debug mode is disabled.
pub trait IsLoggingAllowed {
    type Value: AllowedStatus;
    const LOGGING_ALLOWED: bool = Self::Value::LOGGING_ALLOWED;
    // fn debug_print(&self) -> DbgPrinter {
    //     DbgPrinter::default()
    // }

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
        )*
    };
}

macro_rules! enable_logging {
    ($($name:ty,)* ) => {
        $(
            impl IsLoggingAllowed for $name {
                type Value = Allowed;
                // fn debug_print(&self) -> DbgPrinter {
                //     DbgPrinter {
                //         value1: Some(self),
                //         ..Default::default()
                //     }
                // }

                fn fmt_loggable(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
        )*
    };
}

enable_logging!(
    // Account
    AccountIdInternal,
    AccountIdLight,
    // Media
    ModerationRequestIdDb,
    ModerationRequestId,    // TODO: combine with ModerationRequestIdDb
    ContentIdDb,
    ContentId,
    ImageSlot,
    ModerationId,
    ModerationQueueNumber,
);

disable_logging!(
    // Account
    GoogleAccountId,
    // Media
    ModerationRequestContent,
    PrimaryImage,
    ContentState,
    (),
);


impl <
    T1: IsLoggingAllowed,
    T2: IsLoggingAllowed,
> IsLoggingAllowed for (T1, T2) where Self: Debug {
    type Value = Allowed;
    // fn debug_print<'a>(&'a self) -> DbgPrinter<'a> {
    //     DbgPrinter {
    //         value1: Some(&self.0),
    //         value2: Some(&self.1),
    //         ..Default::default()
    //     }
    // }

    fn fmt_loggable(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;
        self.0.fmt_loggable(f)?;
        f.write_str(", ")?;
        self.1.fmt_loggable(f)?;
        f.write_str(")")?;
        Ok(())
    }
}

impl <
    T1: IsLoggingAllowed + Debug,
    T2: IsLoggingAllowed + Debug,
    T3: IsLoggingAllowed + Debug,
> IsLoggingAllowed for (T1, T2, T3) where Self: Debug {
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

impl <
    T1: IsLoggingAllowed + Debug,
    T2: IsLoggingAllowed + Debug,
    T3: IsLoggingAllowed + Debug,
    T4: IsLoggingAllowed + Debug,
> IsLoggingAllowed for (T1, T2, T3, T4) where Self: Debug {
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
