use libc::c_uint;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use sysinfo::Pid;

use super::{GpuItem, Platform};

use fdinfo::FdInfo;
mod fdinfo;

struct LinuxProcess {
    fdinfos: HashMap<(c_uint, c_uint), FdInfo>,
    gpu_usage: Option<f32>,
    pid: Pid,
    proc_path: PathBuf,
    time: Instant,
    version: u64,
}

impl LinuxProcess {
    fn new(pid: Pid, proc_path: PathBuf) -> Self {
        Self {
            fdinfos: HashMap::new(),
            gpu_usage: None,
            pid,
            proc_path,
            time: Instant::now(),
            version: 0,
        }
    }

    fn update(&mut self, version: u64, nvml: &Box<dyn Platform>) {
        let time = Instant::now();
        let mut fdinfos = FdInfo::for_proc_path(&self.proc_path);
        let duration = time.saturating_duration_since(self.time).as_secs_f32();

        // Add DRM fdinfo GPU usage to NVML GPU usage
        self.gpu_usage = nvml.process_gpu_usage(self.pid);
        for (id, fdinfo) in fdinfos.iter_mut() {
            if let Some(last_fdinfo) = self.fdinfos.get(id) {
                for (name, nanos, usage) in fdinfo.engines.iter_mut() {
                    for (last_name, last_nanos, _) in last_fdinfo.engines.iter() {
                        if last_name == name {
                            *usage = 100.0
                                * Duration::from_nanos(nanos.saturating_sub(*last_nanos))
                                    .as_secs_f32()
                                / duration;
                            //TODO: filter by engine name
                            self.gpu_usage = Some(self.gpu_usage.map_or(*usage, |x| x + *usage));
                        }
                    }
                }
            }
        }

        self.fdinfos = fdinfos;
        self.time = time;
        self.version = version;
    }
}

pub struct LinuxPlatform {
    gpu_items: Vec<GpuItem>,
    nvml: Box<dyn Platform>,
    processes: HashMap<Pid, LinuxProcess>,
    version: u64,
}

impl LinuxPlatform {
    pub fn new() -> Self {
        Self {
            gpu_items: Vec::new(),
            #[cfg(feature = "nvml")]
            nvml: Box::new(super::nvml::NvmlPlatform::new()),
            #[cfg(not(feature = "nvml"))]
            nvml: Box::new(super::FallbackPlatform),
            processes: HashMap::new(),
            version: 0,
        }
    }
}

impl Platform for LinuxPlatform {
    fn refresh_gpus(&mut self) {
        self.nvml.refresh_gpus();

        self.gpu_items.clear();

        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry_res in entries {
                let Ok(entry) = entry_res else { continue };
                let file_name = entry.file_name();
                let Some(name_str) = file_name.to_str() else {
                    continue;
                };
                let Some(id_str) = name_str.strip_prefix("card") else {
                    continue;
                };
                let Ok(id) = id_str.parse::<c_uint>() else {
                    continue;
                };
                let drm_path = entry.path();
                let device_path = drm_path.join("device");

                let mut bus_id = None;
                if let Ok(link_path) = fs::read_link(&device_path) {
                    if let Some(link_name) = link_path.file_name() {
                        bus_id = Some(link_name.to_string_lossy().into());
                    }
                }

                let name_from_pci_ids = || -> Result<String, Box<dyn std::error::Error>> {
                    let vendor_str = fs::read_to_string(device_path.join("vendor"))?;
                    let vendor_id =
                        u16::from_str_radix(vendor_str.trim().trim_start_matches("0x"), 16)?;
                    let device_str = fs::read_to_string(device_path.join("device"))?;
                    let device_id =
                        u16::from_str_radix(device_str.trim().trim_start_matches("0x"), 16)?;
                    if let Some(entry) = pci_ids::Device::from_vid_pid(vendor_id, device_id) {
                        Ok(format!(
                            "{} {}",
                            match vendor_id {
                                0x1002 | 0x1022 => "AMD",
                                0x10DE => "NVIDIA",
                                0x8086 => "Intel",
                                _ => entry.vendor().name(),
                            },
                            entry.name()
                        ))
                    } else {
                        Err(format!("no entry for {:04x}:{:04x}", vendor_id, device_id).into())
                    }
                };

                //TODO: only update name when GPUs change
                let name = match name_from_pci_ids() {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::warn!("failed to get name from PCI IDs: {}", err);
                        format!("Unknown GPU {}", id)
                    }
                };

                let mut gpu_item = GpuItem {
                    bus_id,
                    name,
                    usage: 0.0,
                    vram_used: 0,
                    vram_total: 0,
                };

                //TODO: log errors
                //TODO: gpu_busy_percent is only available on AMD
                if let Ok(data) = fs::read_to_string(device_path.join("gpu_busy_percent")) {
                    gpu_item.usage = data.trim().parse().unwrap_or_default();
                };
                //TODO: mem_info_vram_used is only available on AMD
                if let Ok(data) = fs::read_to_string(device_path.join("mem_info_vram_used")) {
                    gpu_item.vram_used = data.trim().parse().unwrap_or_default();
                };
                //TODO: mem_info_vram_total is only available on AMD
                if let Ok(data) = fs::read_to_string(device_path.join("mem_info_vram_total")) {
                    gpu_item.vram_total = data.trim().parse().unwrap_or_default();
                };

                self.gpu_items.push(gpu_item)
            }
        }

        'nvml_gpus: for nvml_gpu in self.nvml.gpus() {
            if let Some(nvml_bus_id) = &nvml_gpu.bus_id {
                for gpu in self.gpu_items.iter_mut() {
                    if gpu.bus_id.as_ref() == Some(nvml_bus_id) {
                        *gpu = nvml_gpu;
                        continue 'nvml_gpus;
                    }
                }
            }
            self.gpu_items.push(nvml_gpu);
        }
    }

    fn gpus(&self) -> Vec<GpuItem> {
        self.gpu_items.clone()
    }

    fn refresh_processes(&mut self) {
        self.nvml.refresh_processes();

        self.version += 1;
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry_res in entries {
                let Ok(entry) = entry_res else { continue };
                let file_name = entry.file_name();
                let Some(pid_str) = file_name.to_str() else {
                    continue;
                };
                let Ok(pid) = pid_str.parse::<Pid>() else {
                    continue;
                };
                self.processes
                    .entry(pid)
                    .or_insert_with(|| LinuxProcess::new(pid, entry.path()))
                    .update(self.version, &self.nvml)
            }
        }
        self.processes.retain(|_k, v| v.version == self.version)
    }

    fn process_gpu_usage(&self, pid: Pid) -> Option<f32> {
        self.processes.get(&pid)?.gpu_usage
    }
}
