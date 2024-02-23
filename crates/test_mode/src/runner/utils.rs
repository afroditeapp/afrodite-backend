

// Code that might be useful when implementing concurrent server test excecution

/*


//! Common test utilities.

use std::{collections::VecDeque, net::SocketAddrV4};

use config::Config;
use tokio::sync::Mutex;

pub struct ServerHandle {
    server_task: tokio::task::JoinHandle<()>,
    id: server_id::ServerId,
}

impl ServerHandle {
    pub async fn new() -> Self {
        let id = server_id::block_for_next_free_id().await;


        let server_task = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        });
        ServerHandle {
            id,
            server_task,
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        //server_id::blocking_add_id_to_pool(self.id);
    }
}


mod server_id {

    use std::{collections::VecDeque, sync::OnceLock};

    use tokio::sync::Mutex;

    pub struct ServerId {
        id: String,
    }

    impl ServerId {
        pub fn new(id: String) -> Self {
            Self { id }
        }

        pub fn as_str(&self) -> &str {
            &self.id
        }
    }

    fn server_id_pool() -> &'static Mutex<VecDeque<ServerId>> {
        static SERVER_ID_POOL: OnceLock<Mutex<VecDeque<ServerId>>> = OnceLock::new();
        SERVER_ID_POOL.get_or_init(|| {
            let mut data = VecDeque::new();
            for id in 0..1 {
                data.push_back(ServerId::new(id.to_string()));
            }
            Mutex::new(data)
        })
    }

    pub async fn block_for_next_free_id() -> ServerId {
        loop {
            let mut pool = server_id_pool().lock().await;
            if let Some(id) = pool.pop_front() {
                return id;
            } else {
                drop(pool);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }

    pub fn blocking_add_id_to_pool(id: ServerId) {
        let mut pool = server_id_pool().blocking_lock();
        pool.push_back(id);
    }

}

*/
