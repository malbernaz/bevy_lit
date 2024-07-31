#define_import_path bevy_lit::types

struct Lighting2dSettings {
    coc: f32,
    fixed_resolution: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding_0: u32, 
    _webgl2_padding_1: u32, 
#endif
}

struct AmbientLight2d {
    color: vec4<f32>,
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
