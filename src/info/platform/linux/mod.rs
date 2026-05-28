use libc::c_uint;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use sysinfo::Pid;

use super::Platform;

use fdinfo::FdInfo;
mod fdinfo;

struct LinuxProcess {
    fdinfos: HashMap<(c_uint, c_uint), FdInfo>,
    gpu_usage: Option<f32>,
    proc_path: PathBuf,
    time: Instant,
    version: u64,
}

impl LinuxProcess {
    fn new(proc_path: PathBuf) -> Self {
        Self {
            fdinfos: HashMap::new(),
            gpu_usage: None,
            proc_path,
            time: Instant::now(),
            version: 0,
        }
    }

    fn update(&mut self, version: u64) {
        let time = Instant::now();
        let mut fdinfos = FdInfo::for_proc_path(&self.proc_path);
        let duration = time.saturating_duration_since(self.time).as_secs_f32();

        self.gpu_usage = None;
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

#[derive(Default)]
pub struct LinuxPlatform {
    version: u64,
    processes: HashMap<Pid, LinuxProcess>,
}

impl Platform for LinuxPlatform {
    fn refresh_processes(&mut self) {
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
                    .or_insert_with(|| LinuxProcess::new(entry.path()))
                    .update(self.version)
            }
        }
        self.processes.retain(|_k, v| v.version == self.version)
    }

    fn process_gpu_usage(&self, pid: Pid) -> Option<f32> {
        self.processes.get(&pid)?.gpu_usage
    }
}
