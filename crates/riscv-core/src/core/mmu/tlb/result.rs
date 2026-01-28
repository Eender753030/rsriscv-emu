#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlbResult {
    Hit(bool, u32),
    Miss,
    PageFault,
    UpdateAD,
}
