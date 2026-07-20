/// Reference resolution the level and camera projection are designed around. The camera's
/// `ScalingMode::AutoMin` (see `setup.rs`) zooms this area to fill the actual window, so the
/// whole scene grows/shrinks with the window (e.g. bigger on fullscreen) instead of the
/// default 1:1 pixel mapping, which would just reveal more or less of the world.
pub const WORLD_WIDTH: f32 = 1280.0;
pub const WORLD_HEIGHT: f32 = 720.0;
