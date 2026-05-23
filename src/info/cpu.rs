use sysinfo::Cpu;

#[derive(Clone, Debug)]
pub struct CpuItem {
    pub name: String,
    pub cpu_usage: f32,
}

impl CpuItem {
    pub fn new(cpu: &Cpu) -> Self {
        Self {
            name: cpu.name().into(),
            cpu_usage: cpu.cpu_usage(),
        }
    }
}
