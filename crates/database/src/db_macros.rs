#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! define_current_read_commands {
    ($struct_name:ident, $sync_name:ident) => {
        // TODO: Remove struct_name

        pub struct $sync_name<C: database::ConnectionProvider> {
            cmds: C,
        }

        impl<C: database::ConnectionProvider> $sync_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            pub fn conn(&mut self) -> &mut database::DieselConnection {
                self.cmds.conn()
            }

            pub fn read(
                &mut self,
            ) -> crate::current::read::CurrentSyncReadCommands<&mut database::DieselConnection>
            {
                crate::current::read::CurrentSyncReadCommands::new(self.conn())
            }
        }
    };
}

#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! define_current_write_commands {
    ($struct_name:ident, $sync_name:ident) => {
        // TODO: Remove struct_name

        pub struct $sync_name<C: database::ConnectionProvider> {
            cmds: C,
        }

        impl<C: database::ConnectionProvider> $sync_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            pub fn conn(&mut self) -> &mut database::DieselConnection {
                self.cmds.conn()
            }

            // pub fn into_conn(self) -> &'a mut crate::diesel::DieselConnection {
            //     self.cmds.conn
            // }

            pub fn read(
                &mut self,
            ) -> crate::current::read::CurrentSyncReadCommands<&mut database::DieselConnection>
            {
                crate::current::read::CurrentSyncReadCommands::new(self.conn())
            }

            pub fn cmds(
                &mut self,
            ) -> crate::current::write::CurrentSyncWriteCommands<&mut database::DieselConnection>
            {
                crate::current::write::CurrentSyncWriteCommands::new(self.conn())
            }

            pub fn common_read_access(
                &mut self,
            ) -> $crate::current::read::CurrentSyncReadCommands<&mut database::DieselConnection>
            {
                $crate::current::read::CurrentSyncReadCommands::new(self.conn())
            }

            pub fn common_write_access(
                &mut self,
            ) -> $crate::current::write::CurrentSyncWriteCommands<&mut database::DieselConnection>
            {
                $crate::current::write::CurrentSyncWriteCommands::new(self.conn())
            }
        }
    };
}

#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! define_history_read_commands {
    ($struct_name:ident, $sync_name:ident) => {
        // TODO: Remove struct_name

        pub struct $sync_name<C: database::ConnectionProvider> {
            cmds: C,
        }

        impl<C: database::ConnectionProvider> $sync_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            pub fn conn(&mut self) -> &mut database::DieselConnection {
                self.cmds.conn()
            }

            pub fn read(
                conn: &mut database::DieselConnection,
            ) -> crate::current::read::CurrentSyncReadCommands<&mut database::DieselConnection>
            {
                crate::current::read::CurrentSyncReadCommands::new(conn)
            }
        }
    };
}

#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! define_history_write_commands {
    ($struct_name:ident, $sync_name:ident) => {
        // TODO: Remove struct_name

        pub struct $sync_name<C: database::ConnectionProvider> {
            cmds: C,
        }

        impl<C: database::ConnectionProvider> $sync_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            pub fn conn(&mut self) -> &mut database::DieselConnection {
                self.cmds.conn()
            }

            // pub fn into_conn(self) -> &'a mut crate::diesel::DieselConnection {
            //     self.cmds.conn
            // }

            pub fn read(
                conn: &mut database::DieselConnection,
            ) -> crate::current::read::CurrentSyncReadCommands<&mut database::DieselConnection>
            {
                crate::current::read::CurrentSyncReadCommands::new(conn)
            }
        }
    };
}
