#define_import_path bevy_lit::types

struct RaymarchSettings {
    max_steps: u32,
    jitter: f32,
    sharpness: f32,
}

struct Lighting2dSettings {
    // blur circle of confusion diameter
    coc: f32,
    fixed_resolution: u32,
    ambient_light: vec4<f32>,
    raymarch: RaymarchSettings,
}

struct LightOccluder2d {
    center: vec2<f32>,
    half_size: vec2<f32>,
}

struct PointLight2d {
    center: vec2<f32>,
    color: vec4<f32>,
    falloff: f32,
    intensity: f32,
    radius: f32,
}
