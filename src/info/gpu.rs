#[derive(Clone, Debug)]
pub struct GpuItem {
    pub name: String,
    pub usage: f32,
    pub vram_used: u64,
    pub vram_total: u64,
}
