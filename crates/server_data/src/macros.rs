#[macro_export]
macro_rules! define_cmd_wrapper_read {
    ($struct_name:ident) => {
        pub struct $struct_name<'a>(&'a $crate::db_manager::handle_types::ReadHandleType);

        impl<'a> $struct_name<'a> {
            pub fn new(c: &'a $crate::db_manager::handle_types::ReadHandleType) -> Self {
                Self(c)
            }
        }

        impl $crate::db_manager::InternalReading for &$struct_name<'_> {
            fn file_dir(&self) -> &$crate::file::utils::FileDir {
                self.0.file_dir()
            }

            fn current_read_handle(&self) -> &$crate::db_manager::handle_types::CurrentReadHandle {
                self.0.current_read_handle()
            }

            fn history_read_handle(&self) -> &$crate::db_manager::handle_types::HistoryReadHandle {
                self.0.history_read_handle()
            }

            fn cache(&self) -> &$crate::cache::DatabaseCache {
                self.0.cache()
            }

            fn config(&self) -> &$crate::db_manager::handle_types::Config {
                self.0.config()
            }

            fn config_arc(&self) -> std::sync::Arc<$crate::db_manager::handle_types::Config> {
                self.0.config_arc()
            }

            fn profile_attributes(
                &self,
            ) -> &$crate::profile_attributes::ProfileAttributesSchemaManager {
                self.0.profile_attributes()
            }
        }
    };
}

#[macro_export]
macro_rules! define_cmd_wrapper_write {
    ($struct_name:ident) => {
        pub struct $struct_name<'a>(&'a $crate::db_manager::handle_types::WriteHandleType);

        impl<'a> $struct_name<'a> {
            pub fn new(c: &'a $crate::db_manager::handle_types::WriteHandleType) -> Self {
                Self(c)
            }

            pub fn handle(&self) -> &$crate::db_manager::handle_types::WriteHandleType {
                &self.0
            }
        }

        impl $crate::db_manager::InternalWriting for &$struct_name<'_> {
            fn config(&self) -> &$crate::db_manager::handle_types::Config {
                $crate::db_manager::InternalWriting::config(self.0)
            }

            fn config_arc(&self) -> std::sync::Arc<$crate::db_manager::handle_types::Config> {
                $crate::db_manager::InternalWriting::config_arc(self.0)
            }

            fn file_dir(&self) -> &$crate::file::utils::FileDir {
                $crate::db_manager::InternalWriting::file_dir(self.0)
            }

            fn current_write_handle(
                &self,
            ) -> &$crate::db_manager::handle_types::CurrentWriteHandle {
                $crate::db_manager::InternalWriting::current_write_handle(self.0)
            }

            fn history_write_handle(
                &self,
            ) -> &$crate::db_manager::handle_types::HistoryWriteHandle {
                $crate::db_manager::InternalWriting::history_write_handle(self.0)
            }

            fn current_read_handle(&self) -> &$crate::db_manager::handle_types::CurrentReadHandle {
                $crate::db_manager::InternalWriting::current_read_handle(self.0)
            }

            fn history_read_handle(&self) -> &$crate::db_manager::handle_types::HistoryReadHandle {
                $crate::db_manager::InternalWriting::history_read_handle(self.0)
            }

            fn cache(&self) -> &$crate::cache::DatabaseCache {
                $crate::db_manager::InternalWriting::cache(self.0)
            }

            fn location(&self) -> &$crate::index::LocationIndexManager {
                $crate::db_manager::InternalWriting::location(self.0)
            }

            fn push_notification_sender(
                &self,
            ) -> &$crate::db_manager::handle_types::PushNotificationSender {
                $crate::db_manager::InternalWriting::push_notification_sender(self.0)
            }

            fn email_sender(&self) -> &$crate::db_manager::handle_types::EmailSenderImpl {
                $crate::db_manager::InternalWriting::email_sender(self.0)
            }

            fn events(&self) -> $crate::event::EventManagerWithCacheReference<'_> {
                $crate::db_manager::InternalWriting::events(self.0)
            }

            fn profile_attributes(
                &self,
            ) -> &$crate::profile_attributes::ProfileAttributesSchemaManager {
                $crate::db_manager::InternalWriting::profile_attributes(self.0)
            }
        }
    };
}

/// Macro for writing to current database with transaction.
/// Calls await automatically.
///
/// ```ignore
/// use server::DataError;
/// use server::data::write::{define_write_commands, db_transaction};
///
/// define_write_commands!(WriteCommandsTest);
///
/// impl WriteCommandsTest<'_> {
///     pub async fn test(
///         &self,
///     ) -> server::result::Result<(), DataError> {
///         db_transaction!(self, move |mut cmds| {
///             Ok(())
///         })?;
///         Ok(())
///     }
/// }
/// ```
#[macro_export]
macro_rules! db_transaction {
    ($state:expr, move |mut $cmds:ident| $commands:expr) => {{ $crate::IntoDataError::into_error($state.db_transaction(move |mut $cmds| ($commands)).await) }};
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        $crate::IntoDataError::into_error(
            $state.db_transaction_common(move |$cmds| ($commands)).await,
        )
    }};
}

#[macro_export]
macro_rules! db_transaction_history {
    ($state:expr, move |mut $cmds:ident| $commands:expr) => {{
        $crate::IntoDataError::into_error(
            $state
                .db_transaction_history(move |mut $cmds| ($commands))
                .await,
        )
    }};
    ($state:expr, move |$cmds:ident| $commands:expr) => {{
        $crate::IntoDataError::into_error(
            $state
                .db_transaction_history(move |$cmds| ($commands))
                .await,
        )
    }};
}
