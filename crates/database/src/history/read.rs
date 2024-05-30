use simple_backend_database::diesel_db::ConnectionProvider;

// macro_rules! define_read_commands {
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
//         }
//     };
// }

pub struct HistorySyncReadCommands<C: ConnectionProvider> {
    _conn: C,
}

impl<C: ConnectionProvider> HistorySyncReadCommands<C> {
    pub fn new(conn: C) -> Self {
        Self { _conn: conn }
    }
}
