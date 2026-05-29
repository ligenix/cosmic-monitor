use std::{borrow::Cow, cmp::Ordering, collections::HashMap, fmt, time::Duration};

use cosmic::{
    iced::{Alignment, Length},
    widget::{
        Icon,
        table::{ItemCategory, ItemInterface},
    },
};
use sysinfo::{Pid, Process, Users};

use super::{GpuId, Platform};
use crate::fl;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum ProcessCategory {
    #[default]
    Name,
    User,
    PID,
    CPU,
    Memory,
    GPU(GpuId),
    GPUTotal,
    DiskRead,
    DiskWrite,
    DiskTotal,
    Priority,
}

impl ProcessCategory {
    pub fn all() -> &'static [Self] {
        &[
            Self::Name,
            Self::User,
            Self::PID,
            Self::CPU,
            Self::Memory,
            Self::GPUTotal,
            Self::DiskRead,
            Self::DiskWrite,
            //TODO: Self::DiskTotal,
            Self::Priority,
        ]
    }

    pub fn data_align(&self) -> Alignment {
        match self {
            Self::Name | Self::User | Self::Priority => Alignment::Start,
            Self::PID
            | Self::CPU
            | Self::Memory
            | Self::GPU(_)
            | Self::GPUTotal
            | Self::DiskRead
            | Self::DiskWrite
            | Self::DiskTotal => Alignment::End,
        }
    }
}

impl fmt::Display for ProcessCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //TODO: translate
        write!(
            f,
            "{}",
            match self {
                Self::Name => "Name".to_string(),
                Self::User => "User".to_string(),
                Self::PID => "PID".to_string(),
                Self::CPU => fl!("cpu"),
                Self::Memory => fl!("memory"),
                Self::GPU(gpu_id) => format!("GPU {:?}", gpu_id),
                Self::GPUTotal => fl!("gpu"),
                Self::DiskRead => "Disk Read".to_string(),
                Self::DiskWrite => "Disk Write".to_string(),
                Self::DiskTotal => "Disk Total".to_string(),
                Self::Priority => "Priority".to_string(),
            }
        )
    }
}

impl ItemCategory for ProcessCategory {
    fn width(&self) -> Length {
        match self {
            Self::Name => Length::Fill,
            Self::User => Length::Fixed(128.0),
            Self::PID => Length::Fixed(96.0),
            Self::CPU => Length::Fixed(64.0),
            Self::Memory => Length::Fixed(96.0),
            Self::GPU(_) | Self::GPUTotal => Length::Fixed(64.0),
            Self::DiskRead | Self::DiskWrite | Self::DiskTotal => Length::Fixed(96.0),
            Self::Priority => Length::Fixed(96.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessItem {
    pub cpu_usage: u32,
    pub cpu_usage_str: String,
    pub disk_read: u64,
    pub disk_read_str: String,
    pub disk_write: u64,
    pub disk_write_str: String,
    pub disk_total: u64,
    pub disk_total_str: String,
    pub gpu_usages: HashMap<GpuId, (u32, String)>,
    pub gpu_total_usage: Option<u32>,
    pub gpu_total_usage_str: String,
    pub memory: u64,
    pub memory_str: String,
    pub name: String,
    pub pid: Pid,
    pub pid_str: String,
    pub priority: Option<i32>,
    pub username: String,
}

impl ProcessItem {
    pub fn new(
        p: &Process,
        cpu_len: usize,
        platform: &Box<dyn Platform>,
        users: &Users,
        refresh: Duration,
    ) -> Self {
        let cpu_usage = ((p.cpu_usage() / (cpu_len as f32)) * 10.0) as u32;
        let cpu_usage_str = format!("{}.{}%", cpu_usage / 10, cpu_usage % 10);

        let disk_usage = p.disk_usage();
        let disk_read = disk_usage.read_bytes / refresh.as_secs();
        let disk_read_str = format!(
            "{}/s",
            humansize::format_size(disk_read, humansize::DECIMAL)
        );
        let disk_write = disk_usage.written_bytes / refresh.as_secs();
        let disk_write_str = format!(
            "{}/s",
            humansize::format_size(disk_write, humansize::DECIMAL)
        );
        let disk_total = disk_read + disk_write;
        let disk_total_str = format!(
            "{}/s",
            humansize::format_size(disk_total, humansize::DECIMAL)
        );

        let pid = p.pid();
        let pid_str = pid.to_string();

        let mut gpu_total_usage = None;
        let mut gpu_usages = HashMap::new();
        for (gpu_id, usage) in platform.process_gpu_usage(pid) {
            let gpu_usage = (usage * 10.0) as u32;
            let gpu_usage_str = format!("{}.{}%", gpu_usage / 10, gpu_usage % 10);
            gpu_total_usage = Some(gpu_total_usage.map_or(gpu_usage, |x| x + gpu_usage));
            gpu_usages.insert(gpu_id, (gpu_usage, gpu_usage_str));
        }
        let gpu_total_usage_str = gpu_total_usage
            .map(|x| format!("{}.{}%", x / 10, x % 10))
            .unwrap_or_default();

        let memory = p.memory();
        let memory_str = format!("{}", humansize::format_size(memory, humansize::BINARY));

        let mut priority = None;

        #[cfg(unix)]
        if let Some(pid) = rustix::process::Pid::from_raw(p.pid().as_u32() as _) {
            match rustix::process::getpriority_process(Some(pid)) {
                Ok(ok) => {
                    priority = Some(ok);
                }
                Err(err) => {
                    log::warn!("failed to get priority for {}: {}", p.pid(), err);
                }
            }
        }

        let username = match p.user_id() {
            Some(uid) => match users.get_user_by_id(uid) {
                Some(user) => user.name().to_string(),
                None => uid.to_string(),
            },
            None => String::new(),
        };

        Self {
            cpu_usage,
            cpu_usage_str,
            disk_read,
            disk_read_str,
            disk_write,
            disk_write_str,
            disk_total,
            disk_total_str,
            gpu_usages,
            gpu_total_usage,
            gpu_total_usage_str,
            memory,
            memory_str,
            name: p.name().to_string_lossy().into(),
            pid,
            pid_str,
            priority,
            username,
        }
    }

    // Like get_text but without allocation
    pub fn text(&self, category: ProcessCategory) -> &str {
        match category {
            ProcessCategory::Name => &self.name,
            ProcessCategory::User => &self.username,
            ProcessCategory::PID => &self.pid_str,
            ProcessCategory::CPU => &self.cpu_usage_str,
            ProcessCategory::Memory => &self.memory_str,
            ProcessCategory::GPU(gpu_id) => &self
                .gpu_usages
                .get(&gpu_id)
                .map(|x| x.1.as_str())
                .unwrap_or_default(),
            ProcessCategory::GPUTotal => &self.gpu_total_usage_str,
            ProcessCategory::DiskRead => &self.disk_read_str,
            ProcessCategory::DiskWrite => &self.disk_write_str,
            ProcessCategory::DiskTotal => &self.disk_total_str,
            //TODO: translate
            ProcessCategory::Priority => self.priority.map_or("N/A", |x| {
                if x < -7 {
                    "Very high"
                } else if x < -2 {
                    "High"
                } else if x < 3 {
                    "Normal"
                } else if x < 7 {
                    "Low"
                } else {
                    "Very low"
                }
            }),
        }
    }
}

impl ItemInterface<ProcessCategory> for ProcessItem {
    fn get_icon(&self, _category: ProcessCategory) -> Option<Icon> {
        None
    }

    //TODO: Use Cow<'a, str> instead so borrows of strings work
    fn get_text(&self, category: ProcessCategory) -> Cow<'static, str> {
        Cow::Owned(self.text(category).into())
    }

    fn compare(&self, other: &Self, category: ProcessCategory) -> Ordering {
        match category {
            ProcessCategory::Name => self.name.cmp(&other.name),
            ProcessCategory::User => self.username.cmp(&other.username),
            ProcessCategory::PID => self.pid.cmp(&other.pid),
            // These are sorted with higher values at the top
            ProcessCategory::CPU => other.cpu_usage.cmp(&self.cpu_usage),
            ProcessCategory::Memory => other.memory.cmp(&self.memory),
            ProcessCategory::GPU(gpu_id) => {
                let self_usage = self
                    .gpu_usages
                    .get(&gpu_id)
                    .map(|x| x.0)
                    .unwrap_or_default();
                let other_usage = other
                    .gpu_usages
                    .get(&gpu_id)
                    .map(|x| x.0)
                    .unwrap_or_default();
                other_usage.cmp(&self_usage)
            }
            ProcessCategory::GPUTotal => other.gpu_total_usage.cmp(&self.gpu_total_usage),
            ProcessCategory::DiskRead => other.disk_read.cmp(&self.disk_read),
            ProcessCategory::DiskWrite => other.disk_write.cmp(&self.disk_write),
            ProcessCategory::DiskTotal => other.disk_total.cmp(&self.disk_total),
            ProcessCategory::Priority => self.priority.cmp(&other.priority),
        }
    }
}
