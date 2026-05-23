use std::{borrow::Cow, cmp::Ordering, fmt, time::Duration};

use cosmic::{
    iced::{Alignment, Length},
    widget::{
        Icon,
        table::{ItemCategory, ItemInterface},
    },
};
use sysinfo::{Pid, Process, Users};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum ProcessCategory {
    #[default]
    Name,
    User,
    PID,
    CPU,
    Memory,
    GPU,
    DiskRead,
    DiskWrite,
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
            Self::GPU,
            Self::DiskRead,
            Self::DiskWrite,
            Self::Priority,
        ]
    }

    pub fn data_align(&self) -> Alignment {
        match self {
            Self::Name | Self::User | Self::Priority => Alignment::Start,
            Self::PID | Self::CPU | Self::Memory | Self::GPU | Self::DiskRead | Self::DiskWrite => {
                Alignment::End
            }
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
                Self::Name => "Name",
                Self::User => "User",
                Self::PID => "PID",
                Self::CPU => "CPU",
                Self::Memory => "Memory",
                Self::GPU => "GPU",
                Self::DiskRead => "Disk Read",
                Self::DiskWrite => "Disk Write",
                Self::Priority => "Priority",
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
            Self::GPU => Length::Fixed(64.0),
            Self::DiskRead => Length::Fixed(96.0),
            Self::DiskWrite => Length::Fixed(96.0),
            Self::Priority => Length::Fixed(96.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessItem {
    pub cpu_usage: u32,
    pub disk_read: u64,
    pub disk_write: u64,
    pub memory: u64,
    pub name: String,
    pub pid: Pid,
    pub priority: Option<i32>,
    pub username: String,
}

impl ProcessItem {
    pub fn new(p: &Process, users: &Users, refresh: Duration) -> Self {
        let username = match p.user_id() {
            Some(uid) => match users.get_user_by_id(uid) {
                Some(user) => user.name().to_string(),
                None => uid.to_string(),
            },
            None => String::new(),
        };

        let disk_usage = p.disk_usage();

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

        Self {
            cpu_usage: (p.cpu_usage() * 10.0) as u32,
            disk_read: disk_usage.read_bytes / refresh.as_secs(),
            disk_write: disk_usage.written_bytes / refresh.as_secs(),
            memory: p.memory(),
            name: p.name().to_string_lossy().into(),
            pid: p.pid(),
            priority,
            username,
        }
    }
}

impl ItemInterface<ProcessCategory> for ProcessItem {
    fn get_icon(&self, _category: ProcessCategory) -> Option<Icon> {
        None
    }

    //TODO: Use Cow<'a, str> instead so borrows of strings work
    fn get_text(&self, category: ProcessCategory) -> Cow<'static, str> {
        match category {
            ProcessCategory::Name => Cow::Owned(self.name.clone()),
            ProcessCategory::User => Cow::Owned(self.username.clone()),
            ProcessCategory::PID => format!("{}", self.pid).into(),
            ProcessCategory::CPU => {
                format!("{}.{}%", self.cpu_usage / 10, self.cpu_usage % 10).into()
            }
            ProcessCategory::Memory => {
                format!("{}", humansize::format_size(self.memory, humansize::BINARY)).into()
            }
            ProcessCategory::DiskRead => format!(
                "{}/s",
                humansize::format_size(self.disk_read, humansize::DECIMAL)
            )
            .into(),
            ProcessCategory::DiskWrite => format!(
                "{}/s",
                humansize::format_size(self.disk_write, humansize::DECIMAL)
            )
            .into(),
            //TODO: translate
            ProcessCategory::Priority => self.priority.map_or("N/A".into(), |x| {
                if x < -7 {
                    "Very high".into()
                } else if x < -2 {
                    "High".into()
                } else if x < 3 {
                    "Normal".into()
                } else if x < 7 {
                    "Low".into()
                } else {
                    "Very low".into()
                }
            }),
            //TODO
            _ => "TODO".into(),
        }
    }

    fn compare(&self, other: &Self, category: ProcessCategory) -> Ordering {
        match category {
            ProcessCategory::Name => self.name.cmp(&other.name),
            ProcessCategory::User => self.username.cmp(&other.username),
            ProcessCategory::PID => self.pid.cmp(&other.pid),
            // These are sorted with higher values at the top
            ProcessCategory::CPU => other.cpu_usage.cmp(&self.cpu_usage),
            ProcessCategory::Memory => other.memory.cmp(&self.memory),
            ProcessCategory::DiskRead => other.disk_read.cmp(&self.disk_read),
            ProcessCategory::DiskWrite => other.disk_write.cmp(&self.disk_write),
            ProcessCategory::Priority => self.priority.cmp(&other.priority),
            //TODO
            _ => self.pid.cmp(&other.pid),
        }
    }
}
