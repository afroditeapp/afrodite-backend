use std::time::Instant;

use simple_backend_utils::consts::{KIB_IN_BYTES, MIB_IN_BYTES};
use sysinfo::{MemoryRefreshKind, Networks};

pub struct SystemInfo {
    cpu_usage: u32,
    ram_usage_mib: u32,
    network_received_kib_per_second: u32,
    network_transmitted_kib_per_second: u32,
}

impl SystemInfo {
    async fn new(system: &mut Box<sysinfo::System>, networks: &mut NetworkInfo) -> SystemInfo {
        system.refresh_cpu_usage();
        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
        system.refresh_cpu_usage();
        system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let (network_received_kib_per_second, network_transmitted_kib_per_second) =
            networks.get_transfer_rates();
        SystemInfo {
            cpu_usage: system.global_cpu_usage() as u32,
            ram_usage_mib: (system.used_memory() / MIB_IN_BYTES as u64) as u32,
            network_received_kib_per_second,
            network_transmitted_kib_per_second,
        }
    }

    pub fn cpu_usage(&self) -> u32 {
        self.cpu_usage
    }

    pub fn ram_usage_mib(&self) -> u32 {
        self.ram_usage_mib
    }

    pub fn network_received_kib_per_second(&self) -> u32 {
        self.network_received_kib_per_second
    }

    pub fn network_transmitted_kib_per_second(&self) -> u32 {
        self.network_transmitted_kib_per_second
    }
}

/// Network transfer rates calculated from the delta of total transferred
/// bytes between the previous and current sample.
struct NetworkInfo {
    networks: Networks,
    previous_received_bytes: u64,
    previous_transmitted_bytes: u64,
    previous_sample_time: Option<Instant>,
}

impl NetworkInfo {
    fn new() -> Self {
        Self {
            networks: Networks::new(),
            previous_received_bytes: 0,
            previous_transmitted_bytes: 0,
            previous_sample_time: None,
        }
    }

    /// Returns (received, transmitted) in KiB/s. The first call returns zero
    /// because there is no previous sample to calculate a delta from.
    fn get_transfer_rates(&mut self) -> (u32, u32) {
        let sample_time = Instant::now();
        self.networks.refresh(true);

        let mut total_received = 0;
        let mut total_transmitted = 0;
        for (interface_name, data) in self.networks.list() {
            if interface_name.starts_with("lo") {
                continue;
            }
            total_received += data.total_received();
            total_transmitted += data.total_transmitted();
        }

        let Some(previous_sample_time) = self.previous_sample_time else {
            self.previous_received_bytes = total_received;
            self.previous_transmitted_bytes = total_transmitted;
            self.previous_sample_time = Some(sample_time);
            return (0, 0);
        };

        let elapsed_seconds = sample_time
            .duration_since(previous_sample_time)
            .as_secs_f64();

        let received = total_received.saturating_sub(self.previous_received_bytes);
        let transmitted = total_transmitted.saturating_sub(self.previous_transmitted_bytes);

        self.previous_received_bytes = total_received;
        self.previous_transmitted_bytes = total_transmitted;
        self.previous_sample_time = Some(sample_time);

        if elapsed_seconds <= 0.0 {
            return (0, 0);
        }

        let received_rate = (received as f64 / elapsed_seconds / KIB_IN_BYTES as f64) as u64;
        let transmitted_rate = (transmitted as f64 / elapsed_seconds / KIB_IN_BYTES as f64) as u64;

        (
            received_rate.min(u32::MAX as u64) as u32,
            transmitted_rate.min(u32::MAX as u64) as u32,
        )
    }
}

pub struct SystemInfoManager {
    system: Box<sysinfo::System>,
    network_info: NetworkInfo,
}

impl SystemInfoManager {
    pub fn new() -> Self {
        Self {
            system: Box::default(),
            network_info: NetworkInfo::new(),
        }
    }

    pub async fn get_system_info(&mut self) -> SystemInfo {
        SystemInfo::new(&mut self.system, &mut self.network_info).await
    }
}

impl Default for SystemInfoManager {
    fn default() -> Self {
        Self::new()
    }
}
