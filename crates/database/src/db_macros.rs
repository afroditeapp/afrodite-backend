#[macro_export]
macro_rules! define_current_read_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a mut $crate::DieselConnection,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a mut $crate::DieselConnection) -> Self {
                Self { cmds }
            }

            pub fn read(&mut self) -> $crate::DbReadMode<'_> {
                $crate::DbReadMode(self.cmds)
            }

            pub fn conn(&mut self) -> &mut $crate::DieselConnection {
                self.cmds
            }
        }
    };
}

#[macro_export]
macro_rules! define_current_write_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a mut $crate::DieselConnection,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a mut $crate::DieselConnection) -> Self {
                Self { cmds }
            }

            pub fn read(&mut self) -> $crate::DbReadMode<'_> {
                $crate::DbReadMode(self.cmds)
            }

            pub fn write(&mut self) -> $crate::DbWriteMode<'_> {
                $crate::DbWriteMode(self.cmds)
            }

            pub fn conn(&mut self) -> &mut $crate::DieselConnection {
                self.cmds
            }
        }
    };
}

#[macro_export]
macro_rules! define_history_read_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a mut $crate::DieselConnection,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a mut $crate::DieselConnection) -> Self {
                Self { cmds }
            }

            pub fn conn(&mut self) -> &mut $crate::DieselConnection {
                self.cmds
            }

            pub fn read(conn: &mut $crate::DieselConnection) -> $crate::DbReadModeHistory<'_> {
                $crate::DbReadModeHistory(conn)
            }
        }
    };
}

#[macro_export]
macro_rules! define_history_write_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: &'a mut $crate::DieselConnection,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: &'a mut $crate::DieselConnection) -> Self {
                Self { cmds }
            }

            pub fn write(&mut self) -> $crate::DbWriteModeHistory<'_> {
                $crate::DbWriteModeHistory(self.cmds)
            }

            pub fn read(&mut self) -> $crate::DbReadModeHistory<'_> {
                $crate::DbReadModeHistory(self.cmds)
            }

            pub fn conn(&mut self) -> &mut $crate::DieselConnection {
                self.cmds
            }
        }
    };
}

#[macro_export]
macro_rules! eq_optional {
    ($column:expr, $value:expr) => {{
        let value = $value;
        let has_value = value.is_some();

        has_value
            .into_sql::<::diesel::sql_types::Bool>()
            .and($column.eq(value))
            .or((!has_value)
                .into_sql::<::diesel::sql_types::Bool>()
                .and($column.is_null()))
    }};
}
