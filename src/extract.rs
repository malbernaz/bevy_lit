use bevy::{
    prelude::*,
    render::{render_resource::ShaderType, view::ViewVisibility, Extract},
};

use crate::{components::LightOccluder2d, prelude::*, resources::Lighting2dSettings};

#[derive(Component, Clone, ShaderType)]
pub struct ExtractedAmbientLight2d {
    pub color: LinearRgba,
}

#[derive(Component, Clone, ShaderType)]
pub struct ExtractedLighting2dSettings {
    pub blur_coc: f32,
    pub fixed_resolution: u32,
    #[cfg(all(feature = "webgl2", target_arch = "wasm32", not(feature = "webgpu")))]
    _webgl2_padding_0: u32,
    #[cfg(all(feature = "webgl2", target_arch = "wasm32", not(feature = "webgpu")))]
    _webgl2_padding_1: u32,
}

pub fn extract_lighting_resources(
    mut commands: Commands,
    ambient_light: Extract<Res<AmbientLight2d>>,
    lighting_settings: Extract<Res<Lighting2dSettings>>,
    views_query: Extract<Query<Entity, With<Camera2d>>>,
) {
    let bundle = (
        ExtractedAmbientLight2d {
            color: ambient_light.color.to_linear() * ambient_light.brightness,
        },
        ExtractedLighting2dSettings {
            blur_coc: lighting_settings.shadow_softness,
            fixed_resolution: if lighting_settings.fixed_resolution {
                1
            } else {
                0
            },
            #[cfg(all(feature = "webgl2", target_arch = "wasm32", not(feature = "webgpu")))]
            _webgl2_padding_0: Default::default(),
            #[cfg(all(feature = "webgl2", target_arch = "wasm32", not(feature = "webgpu")))]
            _webgl2_padding_1: Default::default(),
        },
    );

    let values = views_query
        .iter()
        .map(|e| (e, bundle.clone()))
        .collect::<Vec<_>>();

    commands.insert_or_spawn_batch(values);
}

#[derive(Component, Default, Clone, ShaderType)]
pub struct ExtractedLightOccluder2d {
    pub center: Vec2,
    pub half_size: Vec2,
}

pub fn extract_light_occluders(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    light_occluders_query: Extract<
        Query<(Entity, &LightOccluder2d, &GlobalTransform, &ViewVisibility)>,
    >,
) {
    let mut values = Vec::with_capacity(*previous_len);

    for (entity, light_occluder, transform, view_visibility) in &light_occluders_query {
        if !view_visibility.get() {
            continue;
        }

        values.push((
            entity,
            ExtractedLightOccluder2d {
                half_size: light_occluder.half_size,
                center: transform.translation().xy(),
            },
        ));
    }

    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

#[derive(Component, Default, Clone, ShaderType)]
pub struct ExtractedPointLight2d {
    pub center: Vec2,
    pub color: LinearRgba,
    pub falloff: f32,
    pub intensity: f32,
    pub radius: f32,
}

pub fn extract_point_lights(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    point_lights_query: Extract<Query<(Entity, &PointLight2d, &GlobalTransform, &ViewVisibility)>>,
) {
    let mut values = Vec::with_capacity(*previous_len);

    for (entity, point_light, transform, visibility) in point_lights_query.iter() {
        if !visibility.get() {
            continue;
        }

        values.push((
            entity,
            ExtractedPointLight2d {
                color: point_light.color.to_linear(),
                center: transform.translation().xy(),
                radius: point_light.radius,
                intensity: point_light.intensity,
                falloff: point_light.falloff,
            },
        ));
    }

    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}
