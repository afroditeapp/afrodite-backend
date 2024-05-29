use server_data::define_server_data_write_commands;

define_server_data_write_commands!(WriteCommandsChatAdmin);
define_db_transaction_command!(WriteCommandsChatAdmin);

impl WriteCommandsChatAdmin<'_> {}
