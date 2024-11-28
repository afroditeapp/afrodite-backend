
#[macro_export]
macro_rules! define_cmd_wrapper_read {
    ($struct_name:ident) => {
        pub struct $struct_name<'a>(&'a $crate::ReadHandleType);

        impl<'a> $struct_name<'a> {
            pub fn new(c: &'a $crate::ReadHandleType) -> Self {
                Self(c)
            }
        }

        impl $crate::db_manager::InternalReading for &$struct_name<'_> {
            fn root(&self) -> &$crate::db_manager::DatabaseRoot {
                self.0.root()
            }

            fn current_read_handle(&self) -> &$crate::db_manager::CurrentReadHandle {
                self.0.current_read_handle()
            }

            fn history_read_handle(&self) -> &$crate::db_manager::HistoryReadHandle {
                self.0.history_read_handle()
            }

            fn cache(&self) -> &$crate::cache::DatabaseCache {
                self.0.cache()
            }
        }
    }
}

#[macro_export]
macro_rules! define_cmd_wrapper {
    ($struct_name:ident) => {
        pub struct $struct_name<C>(C);

        impl<C> $struct_name<C> {
            pub fn new(c: C) -> Self {
                Self(c)
            }
        }

        impl <C> core::ops::Deref for $struct_name<C> {
            type Target = C;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    }
}
