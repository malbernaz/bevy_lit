use bevy::prelude::*;

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct Lighting2dSettings {
    pub shadow_softness: f32,
    #[reflect(ignore)]
    pub viewport: UVec2,
}

impl Default for Lighting2dSettings {
    fn default() -> Self {
        Self {
            shadow_softness: 0.0,
            viewport: UVec2::ZERO,
        }
    }
}

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct AmbientLight2d {
    pub color: Color,
    pub brightness: f32,
}

impl Default for AmbientLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            brightness: 0.8,
        }
    }
}
