use bevy::{
    prelude::*,
    render::{render_resource::ShaderType, view::ViewVisibility, Extract},
};

use crate::{
    components::LightOccluder2d,
    gpu_resources::{
        AmbientLight2dUniform, GpuAmbientLight2d, GpuLighting2dGpuSettings,
        Lighting2dSettingsUniform,
    },
    prelude::*,
    resources::Lighting2dSettings,
};

pub fn extract_lighting_resources(
    mut commands: Commands,
    ambient_light: Extract<Res<AmbientLight2d>>,
    lighting_settings: Extract<Res<Lighting2dSettings>>,
) {
    commands.insert_resource(AmbientLight2dUniform::new(GpuAmbientLight2d {
        color: ambient_light.color.to_linear().to_vec4() * ambient_light.brightness,
    }));

    let Lighting2dSettings {
        shadow_softness,
        viewport,
    } = lighting_settings.clone();
    let UVec2 { x, y } = viewport;

    let viewport_d = ((x + y) as f32).powi(2).sqrt();

    commands.insert_resource(Lighting2dSettingsUniform::new(GpuLighting2dGpuSettings {
        blur_coc: (shadow_softness * viewport_d) / 2000.0,
        viewport,
    }));
}

#[derive(Debug, Component, Default, Clone, ShaderType)]
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

#[derive(Component, Debug, Clone, ShaderType)]
pub struct ExtractedPointLight2d {
    pub center: Vec2,
    pub color: Vec4,
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
                color: point_light.color.to_linear().to_vec4(),
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
