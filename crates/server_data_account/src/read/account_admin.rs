use server_data::define_server_data_read_commands;

define_server_data_read_commands!(ReadCommandsAccountAdmin);
define_db_read_command!(ReadCommandsAccountAdmin);

impl ReadCommandsAccountAdmin<'_> {}
