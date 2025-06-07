use sysinfo::MemoryRefreshKind;

pub struct SystemInfo {
    cpu_usage: u32,
    ram_usage_mib: u32,
}

impl SystemInfo {
    async fn new(system: &mut Box<sysinfo::System>) -> SystemInfo {
        system.refresh_cpu_usage();
        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
        system.refresh_cpu_usage();
        system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        SystemInfo {
            cpu_usage: system.global_cpu_usage() as u32,
            ram_usage_mib: (system.used_memory() / 1024 / 1024) as u32,
        }
    }

    pub fn cpu_usage(&self) -> u32 {
        self.cpu_usage
    }

    pub fn ram_usage_mib(&self) -> u32 {
        self.ram_usage_mib
    }
}

pub struct SystemInfoManager {
    system: Box<sysinfo::System>,
}

impl SystemInfoManager {
    pub fn new() -> Self {
        Self {
            system: Box::default(),
        }
    }

    pub async fn get_system_info(&mut self) -> SystemInfo {
        SystemInfo::new(&mut self.system).await
    }
}

impl Default for SystemInfoManager {
    fn default() -> Self {
        Self::new()
    }
}
