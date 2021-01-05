#[derive(Default,Copy, Clone)]
pub struct Rotation{
    pub(crate) angle: f32,
    pub(crate) sin: f32,
    pub(crate) cos: f32,
    pub(crate) is_rotated: bool,
}