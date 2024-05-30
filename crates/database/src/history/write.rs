use simple_backend_database::diesel_db::{
    ConnectionProvider, DieselConnection, DieselDatabaseError,
};

use crate::TransactionError;

// macro_rules! define_write_commands {
//     ($struct_name:ident, $sync_name:ident) => {
//         // TODO: Remove struct_name

//         pub struct $sync_name<C: simple_backend_database::diesel_db::ConnectionProvider> {
//             cmds: C,
//         }

//         impl<C: simple_backend_database::diesel_db::ConnectionProvider> $sync_name<C> {
//             pub fn new(cmds: C) -> Self {
//                 Self { cmds }
//             }

//             pub fn conn(&mut self) -> &mut simple_backend_database::diesel_db::DieselConnection {
//                 self.cmds.conn()
//             }

//             // pub fn into_conn(self) -> &'a mut crate::diesel::DieselConnection {
//             //     self.cmds.conn
//             // }

//             pub fn read(
//                 conn: &mut simple_backend_database::diesel_db::DieselConnection,
//             ) -> crate::history::read::HistorySyncReadCommands<
//                 &mut simple_backend_database::diesel_db::DieselConnection,
//             > {
//                 crate::history::read::HistorySyncReadCommands::new(conn)
//             }
//         }
//     };
// }

pub struct HistorySyncWriteCommands<C: ConnectionProvider> {
    conn: C,
}

impl<C: ConnectionProvider> HistorySyncWriteCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    // pub fn read(&mut self) -> crate::history::read::HistorySyncReadCommands<&mut DieselConnection> {
    //     self.conn.read()
    // }

    pub fn write(&mut self) -> &mut C {
        &mut self.conn
    }

    pub fn conn(&mut self) -> &mut DieselConnection {
        self.conn.conn()
    }
}

impl HistorySyncWriteCommands<&mut DieselConnection> {
    pub fn transaction<
        F: FnOnce(&mut DieselConnection) -> std::result::Result<T, TransactionError> + 'static,
        T,
    >(
        self,
        transaction_actions: F,
    ) -> error_stack::Result<T, DieselDatabaseError> {
        use diesel::prelude::*;
        self.conn
            .transaction(transaction_actions)
            .map_err(|e| e.into_report())
    }
}
