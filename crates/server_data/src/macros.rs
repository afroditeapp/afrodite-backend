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
            fn root(&self) -> &$crate::db_manager::DatabaseRoot {
                self.0.root()
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
                $crate::db_manager::InternalWriting::config(&self.0)
            }

            fn config_arc(&self) -> std::sync::Arc<$crate::db_manager::handle_types::Config> {
                $crate::db_manager::InternalWriting::config_arc(&self.0)
            }

            fn root(&self) -> &$crate::db_manager::DatabaseRoot {
                $crate::db_manager::InternalWriting::root(&self.0)
            }

            fn current_write_handle(
                &self,
            ) -> &$crate::db_manager::handle_types::CurrentWriteHandle {
                $crate::db_manager::InternalWriting::current_write_handle(&self.0)
            }

            fn history_write_handle(
                &self,
            ) -> &$crate::db_manager::handle_types::HistoryWriteHandle {
                $crate::db_manager::InternalWriting::history_write_handle(&self.0)
            }

            fn current_read_handle(&self) -> &$crate::db_manager::handle_types::CurrentReadHandle {
                $crate::db_manager::InternalWriting::current_read_handle(&self.0)
            }

            fn history_read_handle(&self) -> &$crate::db_manager::handle_types::HistoryReadHandle {
                $crate::db_manager::InternalWriting::history_read_handle(&self.0)
            }

            fn cache(&self) -> &$crate::cache::DatabaseCache {
                $crate::db_manager::InternalWriting::cache(&self.0)
            }

            fn location(&self) -> &$crate::index::LocationIndexManager {
                $crate::db_manager::InternalWriting::location(&self.0)
            }

            fn media_backup(&self) -> &$crate::db_manager::handle_types::MediaBackupHandle {
                $crate::db_manager::InternalWriting::media_backup(&self.0)
            }

            fn push_notification_sender(
                &self,
            ) -> &$crate::db_manager::handle_types::PushNotificationSender {
                $crate::db_manager::InternalWriting::push_notification_sender(&self.0)
            }

            fn email_sender(&self) -> &$crate::db_manager::handle_types::EmailSenderImpl {
                $crate::db_manager::InternalWriting::email_sender(&self.0)
            }
        }
    };
}
