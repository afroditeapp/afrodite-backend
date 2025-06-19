use std::sync::atomic::{AtomicU32, Ordering};

macro_rules! create_connection_tracker {
    ($static_variable:ident, $type_name:ident $(,)?) => {
        static $static_variable: AtomicU32 = AtomicU32::new(0);

        pub struct $type_name(());

        impl $type_name {
            pub fn create() -> Self {
                $static_variable.fetch_add(1, Ordering::Relaxed);
                Self(())
            }

            pub fn connection_count() -> u32 {
                $static_variable.load(Ordering::Relaxed)
            }
        }

        impl Drop for $type_name {
            fn drop(&mut self) {
                $static_variable.fetch_sub(1, Ordering::Relaxed);
            }
        }

        impl From<$type_name> for ConnectionTracker {
            fn from(v: $type_name) -> Self {
                Self::$type_name(v)
            }
        }
    };
}

create_connection_tracker!(CONNECTIONS, Connections,);
create_connection_tracker!(CONNECTIONS_MEN, ConnectionsMen,);
create_connection_tracker!(CONNECTIONS_WOMEN, ConnectionsWomen,);
create_connection_tracker!(CONNECTIONS_NONBINARIES, ConnectionsNonbinaries,);

create_connection_tracker!(BOT_CONNECTIONS, BotConnections,);
create_connection_tracker!(BOT_CONNECTIONS_MEN, BotConnectionsMen,);
create_connection_tracker!(BOT_CONNECTIONS_WOMEN, BotConnectionsWomen,);
create_connection_tracker!(BOT_CONNECTIONS_NONBINARIES, BotConnectionsNonbinaries,);

pub enum ConnectionTracker {
    Connections(Connections),
    ConnectionsMen(ConnectionsMen),
    ConnectionsWomen(ConnectionsWomen),
    ConnectionsNonbinaries(ConnectionsNonbinaries),
    BotConnections(BotConnections),
    BotConnectionsMen(BotConnectionsMen),
    BotConnectionsWomen(BotConnectionsWomen),
    BotConnectionsNonbinaries(BotConnectionsNonbinaries),
}
