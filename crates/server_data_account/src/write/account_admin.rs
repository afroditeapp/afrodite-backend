use server_data::define_server_data_write_commands;

define_server_data_write_commands!(WriteCommandsAccountAdmin);
define_db_read_command!(WriteCommandsAccountAdmin);
define_db_transaction_command!(WriteCommandsAccountAdmin);

impl WriteCommandsAccountAdmin<'_> {}
