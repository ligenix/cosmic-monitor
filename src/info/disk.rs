use sysinfo::Disk;

#[derive(Clone, Debug)]
pub struct DiskItem {
    pub name: String,
    pub used: u64,
    pub total: u64,
}

impl DiskItem {
    pub fn new(disk: &Disk) -> Self {
        Self {
            name: disk.name().to_string_lossy().into(),
            used: disk.total_space() - disk.available_space(),
            total: disk.total_space(),
        }
    }
}
