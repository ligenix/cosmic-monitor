use nvml_wrapper::{Nvml, error::NvmlError, struct_wrappers::device::ProcessUtilizationSample};
use std::collections::HashMap;
use sysinfo::Pid;

use super::{GpuItem, Platform};

pub struct NvmlPlatform {
    gpu_items: Vec<GpuItem>,
    nvml: Option<Nvml>,
    processes: HashMap<Pid, HashMap<u32, ProcessUtilizationSample>>,
}

impl NvmlPlatform {
    pub fn new() -> Self {
        Self {
            gpu_items: Vec::new(),
            //TODO: only use NVML if GPU is awake
            //TODO: log error?
            nvml: Nvml::init().ok(),
            processes: HashMap::new(),
        }
    }

    fn refresh_gpus_inner(&mut self) -> Result<(), NvmlError> {
        self.gpu_items.clear();
        let Some(nvml) = &self.nvml else {
            return Ok(());
        };
        for index in 0..nvml.device_count()? {
            let device = nvml.device_by_index(index)?;
            let name = device.name()?;
            let memory_info = device.memory_info()?;
            let util = device.utilization_rates()?;
            self.gpu_items.push(GpuItem {
                name,
                usage: util.gpu as f32,
                vram_used: memory_info.used,
                vram_total: memory_info.total,
            });
        }
        Ok(())
    }

    fn refresh_processes_inner(&mut self) -> Result<(), NvmlError> {
        self.processes.clear();
        let Some(nvml) = &self.nvml else {
            return Ok(());
        };
        for index in 0..nvml.device_count()? {
            let device = nvml.device_by_index(index)?;
            //TODO: last_seen_timestamp
            for sample in device.process_utilization_stats(None)? {
                let pid = Pid::from_u32(sample.pid);
                self.processes
                    .entry(pid)
                    .or_insert_with(|| HashMap::new())
                    .insert(index, sample);
            }
        }
        Ok(())
    }
}

impl Platform for NvmlPlatform {
    fn refresh_gpus(&mut self) {
        //TODO: log error?
        let _ = self.refresh_gpus_inner();
    }

    fn gpus(&self) -> Vec<GpuItem> {
        self.gpu_items.clone()
    }

    fn refresh_processes(&mut self) {
        //TODO: log error?
        let _ = self.refresh_processes_inner();
    }

    fn process_gpu_usage(&self, pid: Pid) -> Option<f32> {
        let samples = self.processes.get(&pid)?;
        //TODO: use more sample information, show each GPU independently
        Some(
            samples
                .iter()
                .fold(0.0, |total, (_index, sample)| total + sample.sm_util as f32),
        )
    }
}
