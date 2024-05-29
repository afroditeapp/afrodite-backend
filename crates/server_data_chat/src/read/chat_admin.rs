use server_data::define_server_data_read_commands;

define_server_data_read_commands!(ReadCommandsChatAdmin);
define_db_read_command!(ReadCommandsChatAdmin);

impl ReadCommandsChatAdmin<'_> {}
