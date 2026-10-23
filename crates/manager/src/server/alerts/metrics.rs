//! System metric collection for alerts.
//!
//! All metrics are collected with the `sysinfo` library.

use sysinfo::{Disks, MemoryRefreshKind, Networks, ProcessesToUpdate, System};

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Disk space usage percentage of the disk with the highest usage.
    pub disk_space_percent: u32,
    /// RAM usage percentage.
    pub ram_percent: u32,
    /// Combined CPU usage percentage (all cores).
    pub cpu_percent: u32,
    /// Highest single core CPU usage percentage.
    pub cpu_per_core_percent: u32,
    /// Index of the core with the highest usage.
    pub cpu_per_core_index: usize,
    /// Processes using the most CPU, highest usage first, maximum of four.
    pub top_processes: Vec<(String, u32)>,
    /// Total received bytes per second across non-loopback interfaces.
    pub received_bytes_per_second: u64,
    /// Total transmitted bytes per second across non-loopback interfaces.
    pub transmitted_bytes_per_second: u64,
}

pub struct MetricsCollector {
    system: Box<System>,
    disks: Disks,
    networks: Networks,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self {
            system: Box::default(),
            disks: Disks::new(),
            networks: Networks::new(),
        }
    }
}

impl MetricsCollector {
    /// Collect current metrics. The first call returns network values of zero
    /// because there is no previous sample to calculate a delta from.
    pub async fn collect(&mut self) -> SystemMetrics {
        // CPU usage requires two refreshes with a minimum interval between them.
        // Reuse the previous check's refresh as the first sample when available.
        self.system.refresh_cpu_usage();
        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
        self.system.refresh_cpu_usage();
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        self.disks.refresh(true);
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        let ram_percent = Self::ram_percent(&self.system);
        let disk_space_percent = Self::disk_space_percent(&self.disks);
        let cpu_percent = self.system.global_cpu_usage() as u32;

        let (cpu_per_core_percent, cpu_per_core_index) = self
            .system
            .cpus()
            .iter()
            .enumerate()
            .map(|(index, cpu)| (cpu.cpu_usage() as u32, index))
            .max_by_key(|(usage, _)| *usage)
            .unwrap_or((0, 0));

        let mut top_processes: Vec<(String, u32)> = self
            .system
            .processes()
            .values()
            .map(|p| (p.name().to_string_lossy().to_string(), p.cpu_usage() as u32))
            .collect();
        top_processes
            .sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top_processes.truncate(4);

        let (received_bytes_per_second, transmitted_bytes_per_second) =
            self.network_bytes_per_second();

        SystemMetrics {
            disk_space_percent,
            ram_percent,
            cpu_percent,
            cpu_per_core_percent,
            cpu_per_core_index,
            top_processes,
            received_bytes_per_second,
            transmitted_bytes_per_second,
        }
    }

    fn ram_percent(system: &System) -> u32 {
        let total = system.total_memory();
        if total == 0 {
            return 0;
        }
        ((system.used_memory() * 100) / total) as u32
    }

    /// Usage percentage of the disk with the largest total space.
    fn disk_space_percent(disks: &Disks) -> u32 {
        disks
            .list()
            .iter()
            .max_by_key(|disk| disk.total_space())
            .map(|disk| {
                let total = disk.total_space();
                #[allow(clippy::manual_checked_ops)]
                if total == 0 {
                    0
                } else {
                    ((total.saturating_sub(disk.available_space()) * 100) / total) as u32
                }
            })
            .unwrap_or(0)
    }

    /// Network transfer rate between the previous and current refresh,
    /// calculated by the `sysinfo` library.
    fn network_bytes_per_second(&mut self) -> (u64, u64) {
        self.networks.refresh(true);

        let mut total_received = 0;
        let mut total_transmitted = 0;
        for (interface_name, data) in self.networks.list() {
            if interface_name.starts_with("lo") {
                continue;
            }
            total_received += data.received();
            total_transmitted += data.transmitted();
        }

        (total_received, total_transmitted)
    }
}
