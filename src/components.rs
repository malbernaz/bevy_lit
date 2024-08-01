use bevy::{
    math::Vec2,
    prelude::*,
    reflect::Reflect,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    transform::components::{GlobalTransform, Transform},
};

#[derive(Component, Clone, Reflect)]
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

#[derive(Component, Clone, Reflect)]
pub struct PointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
}

impl Default for PointLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
            radius: 64.0,
            falloff: 1.0,
        }
    }
}

#[derive(Bundle, Default)]
pub struct PointLight2dBundle {
    pub point_light: PointLight2d,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Component, Default, Clone, Reflect)]
pub struct LightOccluder2d {
    pub half_size: Vec2,
}

impl LightOccluder2d {
    pub fn new(half_size: Vec2) -> Self {
        Self { half_size }
    }
}

#[derive(Bundle, Default)]
pub struct LightOccluder2dBundle {
    pub light_occluder: LightOccluder2d,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}
