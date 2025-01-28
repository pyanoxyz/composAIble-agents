use log::{ debug, info };
use sysinfo::System;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SystemMemory {
    sys: Arc<RwLock<System>>,
}

impl SystemMemory {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all(); // Initial refresh

        Self {
            sys: Arc::new(RwLock::new(sys)),
        }
    }

    /// Returns available memory in gigabytes
    pub async fn get_available_gb(&self) -> f32 {
        let mut sys = self.sys.write().await;
        sys.refresh_all();

        let total_kb = sys.total_memory();
        let used_kb = sys.used_memory();
        let available_kb = total_kb - used_kb;

        debug!(
            "Memory stats (KB): total={}, used={}, available={}",
            total_kb,
            used_kb,
            available_kb
        );

        let available_gb = (available_kb as f64) / (1024.0 * 1024.0 * 1024.0);
        debug!("Available memory: {:.2} GB", available_gb);
        available_gb as f32
    }

    /// Returns total memory in gigabytes
    pub async fn get_total_gb(&self) -> f32 {
        let mut sys = self.sys.write().await;
        sys.refresh_all();

        let total_kb = sys.total_memory();
        let total_gb = (total_kb as f64) / (1024.0 * 1024.0 * 1024.0);
        debug!("Total memory: {:.2} GB", total_gb);
        total_gb as f32
    }

    /// Returns used memory in gigabytes
    pub async fn get_used_gb(&self) -> f32 {
        let mut sys = self.sys.write().await;
        sys.refresh_all();

        let used_kb = sys.used_memory();
        let used_gb = (used_kb as f64) / (1024.0 * 1024.0 * 1024.0);
        debug!("Used memory: {:.2} GB", used_gb);
        used_gb as f32
    }

    /// Returns memory usage as a percentage
    pub async fn get_usage_percentage(&self) -> f32 {
        let mut sys = self.sys.write().await;
        sys.refresh_all();

        let total = sys.total_memory() as f64;
        let used = sys.used_memory() as f64;
        let percentage = (used / total) * 100.0;
        debug!("Memory usage: {:.1}%", percentage);
        percentage as f32
    }

    /// Checks if there's enough memory available for the requested amount
    pub async fn has_available_memory(&self, required_gb: f32) -> bool {
        let available = self.get_available_gb().await;
        debug!("Memory check: {:.2} GB available, {:.2} GB required", available, required_gb);
        available >= required_gb
    }

    /// Get memory status summary
    pub async fn get_memory_status(&self) -> MemoryStatus {
        let mut sys = self.sys.write().await;
        sys.refresh_all();

        let total_kb = sys.total_memory();
        let used_kb = sys.used_memory();
        let available_kb = total_kb - used_kb;

        let status = MemoryStatus {
            total_gb: ((total_kb as f64) / (1024.0 * 1024.0 * 1024.0)) as f32,
            available_gb: ((available_kb as f64) / (1024.0 * 1024.0)) as f32,
            used_gb: ((used_kb as f64) / (1024.0 * 1024.0 * 1024.0)) as f32,
            usage_percentage: (((used_kb as f64) / (total_kb as f64)) * 100.0) as f32,
        };

        debug!("Memory status: {:?}", status);
        status
    }
}

impl SystemMemory {
    pub async fn debug_memory_info(&self) {
        let mut sys = self.sys.write().await;
        sys.refresh_all();

        let total_gb = (sys.total_memory() as f64) / (1024.0 * 1024.0 * 1024.0);
        let used_gb = (sys.used_memory() as f64) / (1024.0 * 1024.0 * 1024.0);
        let available_gb = total_gb - used_gb;
        info!("");
        info!("=== Memory Debug Information ===");
        info!("Total memory (GB): {:.2}", total_gb);
        info!("Used memory (GB): {:.2}", used_gb);
        info!("Available memory (GB): {:.2}", available_gb);
        info!("Memory usage (%): {:.1}", (used_gb / total_gb) * 100.0);
        info!("==============================");
        info!("");
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStatus {
    pub total_gb: f32,
    pub available_gb: f32,
    pub used_gb: f32,
    pub usage_percentage: f32,
}
