use bevy::{
    math::Vec2,
    prelude::*,
    reflect::Reflect,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    transform::components::{GlobalTransform, Transform},
};

/// Represents ambient light in a 2D environment. This component belongs to a [`Camera2d`] entity.
#[derive(Component, Clone, Reflect)]
pub struct AmbientLight2d {
    /// The color of the ambient light.
    pub color: Color,
    /// The brightness of the ambient light.
    pub brightness: f32,
}

impl Default for AmbientLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            brightness: 1.0,
        }
    }
}

/// Settings for 2D lighting. This component belongs to a [`Camera2d`] entity.
#[derive(Component, Clone, Reflect)]
pub struct Lighting2dSettings {
    /// The softness of the shadows.
    pub shadow_softness: f32,
    /// If false, the shadow softness is calculated in relation to the viewport size.
    pub fixed_resolution: bool,
}

impl Default for Lighting2dSettings {
    fn default() -> Self {
        Self {
            shadow_softness: 0.0,
            fixed_resolution: true,
        }
    }
}

/// Represents a point light in a 2D environment.
#[derive(Component, Clone, Reflect)]
pub struct PointLight2d {
    /// The color of the point light.
    pub color: Color,
    /// The intensity of the point light.
    pub intensity: f32,
    /// The radius of the point light's influence.
    pub radius: f32,
    /// The falloff rate of the point light.
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

/// A bundle of components representing a point light in a 2D environment.
#[derive(Bundle, Default)]
pub struct PointLight2dBundle {
    /// The point light component.
    pub point_light: PointLight2d,
    /// The transform component.
    pub transform: Transform,
    /// The global transform component.
    pub global_transform: GlobalTransform,
    /// The visibility component.
    pub visibility: Visibility,
    /// The inherited visibility component.
    pub inherited_visibility: InheritedVisibility,
    /// The view visibility component.
    pub view_visibility: ViewVisibility,
}

/// Represents an occluder that blocks light in a 2D environment.
#[derive(Component, Default, Clone, Reflect)]
pub struct LightOccluder2d {
    /// Half the size of the occluder AABB rectangle.
    pub half_size: Vec2,
}

impl LightOccluder2d {
    pub fn new(half_size: Vec2) -> Self {
        Self { half_size }
    }
}

/// A bundle of components representing a light occluder in a 2D environment.
#[derive(Bundle, Default)]
pub struct LightOccluder2dBundle {
    /// The light occluder component.
    pub light_occluder: LightOccluder2d,
    /// The transform component.
    pub transform: Transform,
    /// The global transform component.
    pub global_transform: GlobalTransform,
    /// The visibility component.
    pub visibility: Visibility,
    /// The inherited visibility component.
    pub inherited_visibility: InheritedVisibility,
    /// The view visibility component.
    pub view_visibility: ViewVisibility,
}
