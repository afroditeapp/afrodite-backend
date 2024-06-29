#[macro_export]
macro_rules! define_server_data_read_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<C: $crate::read::ReadCommandsProvider> {
            cmds: C,
        }

        impl<C: $crate::read::ReadCommandsProvider> $struct_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn cache(&self) -> &$crate::cache::DatabaseCache {
                &self.cmds.read_cmds().cache
            }

            #[allow(dead_code)]
            fn files(&self) -> &$crate::file::utils::FileDir {
                &self.cmds.read_cmds().files
            }

            pub async fn db_read_raw<
                T: FnOnce(
                        &mut $crate::DieselConnection,
                    ) -> error_stack::Result<R, $crate::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, $crate::DieselDatabaseError> {
                self.cmds.read_cmds().db_read_raw(cmd).await
            }

            pub async fn db_read_common<
                T: FnOnce(
                        $crate::CurrentSyncReadCommands<&mut $crate::DieselConnection>,
                    ) -> error_stack::Result<R, $crate::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, $crate::DieselDatabaseError> {
                self.cmds.read_cmds().db_read(cmd).await
            }

            // TODO: change cache operation to return Result?
            pub async fn read_cache<T, Id: Into<model::AccountId>>(
                &self,
                id: Id,
                cache_operation: impl Fn(&$crate::cache::CacheEntry) -> T,
            ) -> error_stack::Result<T, $crate::cache::CacheError> {
                self.cache().read_cache(id, cache_operation).await
            }
        }
    };
}

#[macro_export]
macro_rules! define_server_data_write_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<C: $crate::write::WriteCommandsProvider> {
            cmds: C,
        }

        impl<C: $crate::write::WriteCommandsProvider> $struct_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn cache(&self) -> &$crate::cache::DatabaseCache {
                &self.cmds.write_cmds().cache
            }

            #[allow(dead_code)]
            fn events(&self) -> $crate::event::EventManagerWithCacheReference<'_> {
                $crate::event::EventManagerWithCacheReference::new(
                    &self.cmds.write_cmds().cache,
                    &self.cmds.write_cmds().push_notification_sender,
                )
            }

            #[allow(dead_code)]
            fn config(&self) -> &config::Config {
                &self.cmds.write_cmds().config
            }

            #[allow(dead_code)]
            fn config_arc(&self) -> &std::sync::Arc<config::Config> {
                &self.cmds.write_cmds().config
            }

            #[allow(dead_code)]
            fn file_dir(&self) -> &$crate::file::utils::FileDir {
                &self.cmds.write_cmds().file_dir
            }

            #[allow(dead_code)]
            fn location(&self) -> $crate::index::LocationIndexWriteHandle<'_> {
                $crate::index::LocationIndexWriteHandle::new(&self.cmds.write_cmds().location_index)
            }

            #[allow(dead_code)]
            fn location_iterator(&self) -> $crate::index::LocationIndexIteratorHandle<'_> {
                $crate::index::LocationIndexIteratorHandle::new(
                    &self.cmds.write_cmds().location_index,
                )
            }

            #[allow(dead_code)]
            fn media_backup(&self) -> &simple_backend::media_backup::MediaBackupHandle {
                &self.cmds.write_cmds().media_backup
            }

            #[allow(dead_code)]
            fn common(
                &self,
            ) -> $crate::write::common::WriteCommandsCommon<&$crate::write::WriteCommands> {
                $crate::write::common::WriteCommandsCommon::new(self.cmds.write_cmds())
            }

            pub async fn db_transaction_common<
                T: FnOnce(
                        $crate::CurrentSyncWriteCommands<&mut $crate::DieselConnection>,
                    ) -> error_stack::Result<R, $crate::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, $crate::DieselDatabaseError> {
                self.cmds.write_cmds().db_transaction_common(cmd).await
            }

            pub async fn db_read_raw<
                T: FnOnce(
                        &mut $crate::DieselConnection,
                    ) -> error_stack::Result<R, $crate::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, $crate::DieselDatabaseError> {
                self.cmds.write_cmds().db_read_raw(cmd).await
            }

            pub async fn db_read_common<
                T: FnOnce(
                        $crate::CurrentSyncReadCommands<&mut $crate::DieselConnection>,
                    ) -> error_stack::Result<R, $crate::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, $crate::DieselDatabaseError> {
                self.cmds.write_cmds().db_read(cmd).await
            }

            pub async fn write_cache<T, Id: Into<model::AccountId>>(
                &self,
                id: Id,
                cache_operation: impl FnOnce(
                    &mut $crate::cache::CacheEntry,
                )
                    -> error_stack::Result<T, $crate::cache::CacheError>,
            ) -> error_stack::Result<T, $crate::cache::CacheError> {
                self.cache().write_cache(id, cache_operation).await
            }
        }
    };
}
