#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

macro_rules! define_db_read_command {
    ($struct_name:ident) => {
        impl<C: server_data::read::ReadCommandsProvider> $struct_name<C> {
            pub async fn db_read<
                T: FnOnce(
                        database_profile::current::read::CurrentSyncReadCommands<
                            &mut server_data::DieselConnection,
                        >,
                    ) -> error_stack::Result<R, server_data::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, server_data::DieselDatabaseError> {
                self.db_read_raw(|conn| {
                    cmd(database_profile::current::read::CurrentSyncReadCommands::new(conn))
                })
                .await
            }
        }
    };
}

macro_rules! define_db_read_history_command {
    ($struct_name:ident) => {
        impl<C: server_data::read::ReadCommandsProvider> $struct_name<C> {
            pub async fn db_read_history<
                T: FnOnce(
                        database_profile::history::read::HistorySyncReadCommands<
                            &mut server_data::DieselConnection,
                        >,
                    ) -> error_stack::Result<R, server_data::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, server_data::DieselDatabaseError> {
                self.db_read_history_raw(|conn| {
                    cmd(database_profile::history::read::HistorySyncReadCommands::new(conn))
                })
                .await
            }
        }
    };
}

macro_rules! define_db_read_command_for_write {
    ($struct_name:ident) => {
        impl<C: server_data::write::WriteCommandsProvider> $struct_name<C> {
            pub async fn db_read<
                T: FnOnce(
                        database_profile::current::read::CurrentSyncReadCommands<
                            &mut server_data::DieselConnection,
                        >,
                    ) -> error_stack::Result<R, server_data::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, server_data::DieselDatabaseError> {
                self.db_read_raw(|conn| {
                    cmd(database_profile::current::read::CurrentSyncReadCommands::new(conn))
                })
                .await
            }
        }
    };
}

macro_rules! define_db_transaction_command {
    ($struct_name:ident) => {
        impl<C: server_data::write::WriteCommandsProvider> $struct_name<C> {
            pub async fn db_transaction<
                T: FnOnce(
                        database_profile::current::write::CurrentSyncWriteCommands<
                            &mut server_data::DieselConnection,
                        >,
                    ) -> error_stack::Result<R, server_data::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, server_data::DieselDatabaseError> {
                self.cmds
                    .write_cmds()
                    .db_transaction_raw(|conn| {
                        cmd(database_profile::current::write::CurrentSyncWriteCommands::new(conn))
                    })
                    .await
            }
        }
    };
}

macro_rules! define_db_transaction_history_command {
    ($struct_name:ident) => {
        impl<C: server_data::write::WriteCommandsProvider> $struct_name<C> {
            pub async fn db_transaction_history<
                T: FnOnce(
                        database_profile::history::write::HistorySyncWriteCommands<
                            &mut server_data::DieselConnection,
                        >,
                    ) -> error_stack::Result<R, server_data::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, server_data::DieselDatabaseError> {
                self.cmds
                    .write_cmds()
                    .db_transaction_history_raw(|conn| {
                        cmd(database_profile::history::write::HistorySyncWriteCommands::new(conn))
                    })
                    .await
            }
        }
    };
}

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

macro_rules! db_transaction_history {
    ($state:expr, move |mut $cmds:ident| $commands:expr) => {{
        server_common::data::IntoDataError::into_error(
            $state.db_transaction_history(move |mut $cmds| ($commands)).await,
        )
    }};
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        $crate::data::IntoDataError::into_error(
            $state.db_transaction_history(move |$cmds| ($commands)).await,
        )
    }};
}

pub mod read;
pub mod write;
pub mod statistics;
pub mod app;
