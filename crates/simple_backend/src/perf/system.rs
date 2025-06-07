use sysinfo::MemoryRefreshKind;
use tracing::error;

pub struct SystemInfo {
    cpu_usage: u32,
    ram_usage_mib: u32,
}

impl SystemInfo {
    fn new(mut system: Box<sysinfo::System>) -> (Box<sysinfo::System>, SystemInfo) {
        system.refresh_cpu_usage();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();
        system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let info = SystemInfo {
            cpu_usage: system.global_cpu_usage() as u32,
            ram_usage_mib: (system.used_memory() / 1024 / 1024) as u32,
        };
        (system, info)
    }

    pub fn cpu_usage(&self) -> u32 {
        self.cpu_usage
    }

    pub fn ram_usage_mib(&self) -> u32 {
        self.ram_usage_mib
    }
}

pub struct SystemInfoManager {
    system: Option<Box<sysinfo::System>>,
}

impl SystemInfoManager {
    pub fn new() -> Self {
        Self {
            system: None,
        }
    }

    pub async fn get_system_info(&mut self) -> Option<SystemInfo> {
        let system = self.system.take().unwrap_or_default();

        let result = tokio::task::spawn_blocking(|| {
            SystemInfo::new(system)
        }).await;

        match result {
            Ok((system, info)) => {
                self.system = Some(system);
                Some(info)
            }
            Err(e) => {
                error!("Getting system info failed: {e}");
                None
            }
        }
    }
}

impl Default for SystemInfoManager {
    fn default() -> Self {
        Self::new()
    }
}
