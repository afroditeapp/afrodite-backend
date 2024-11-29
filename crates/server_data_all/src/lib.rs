#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

macro_rules! db_transaction {
    ($state:expr, move |mut $cmds:ident| $commands:expr) => {{
        server_common::data::IntoDataError::into_error(
            $state.db_transaction(move |mut $cmds| ($commands)).await,
        )
    }};
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        $crate::data::IntoDataError::into_error(
            $state.db_transaction(move |$cmds| ($commands)).await,
        )
    }};
}

pub mod load;
pub mod register;
pub mod unlimited_likes;
pub mod app;
