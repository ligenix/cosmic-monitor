use std::collections::HashMap;
use sysinfo::{Components, Disk, Pid};

use super::{GpuId, GpuItem};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(feature = "nvml")]
mod nvml;

pub trait Platform: Send + Sync {
    fn refresh(&mut self, _components: &Components, _refresh_processes: bool) {}

    fn disk_name(&self, _disk: &Disk) -> Option<String> {
        None
    }

    fn gpus(&self) -> Vec<GpuItem> {
        Vec::new()
    }

    fn process_gpu_usage(&self, _pid: Pid) -> HashMap<GpuId, (f32, u64)> {
        HashMap::new()
    }
}

pub struct FallbackPlatform;

impl Platform for FallbackPlatform {}

#[allow(unreachable_code)]
pub fn default_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "linux")]
    return Box::new(linux::LinuxPlatform::new());

    #[cfg(feature = "nvml")]
    return Box::new(nvml::NvmlPlatform::new());

    #[allow(unreachable_code)]
    Box::new(FallbackPlatform)
}
