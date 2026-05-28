#[derive(Clone, Debug)]
pub struct GpuItem {
    pub bus_id: Option<String>,
    pub name: String,
    pub usage: f32,
    pub vram_used: u64,
    pub vram_total: u64,
}
