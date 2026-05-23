use sysinfo::System;

#[derive(Clone, Debug)]
pub struct MemoryItem {
    pub used: u64,
    pub total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
}

impl MemoryItem {
    pub fn new(sys: &System) -> Self {
        //TODO: Cached memory by comparing available_memory and free_memory
        Self {
            used: sys.used_memory(),
            total: sys.total_memory(),
            swap_used: sys.used_swap(),
            swap_total: sys.total_swap(),
        }
    }
}
