#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_lit::{
    types::{Lighting2dSettings, PointLight2d},
    view_transformations::{
        frag_coord_to_ndc,
        position_ndc_to_world,
        position_world_to_ndc,
        ndc_to_uv,
    }
}

@group(0) @binding(1) var<uniform> settings: Lighting2dSettings;

#if AVAILABLE_STORAGE_BUFFER_BINDINGS >= 6
    @group(0) @binding(2) var<storage> lights: array<PointLight2d>;
#else
    const MAX_LIGHTS: u32 = 82u;

    @group(0) @binding(2) var<uniform> lights: array<PointLight2d, MAX_LIGHTS>;
#endif

@group(0) @binding(3) var sdf: texture_2d<f32>;
@group(0) @binding(4) var sdf_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pos = position_ndc_to_world(frag_coord_to_ndc(in.position)).xy;

    var lighting_color = vec4(settings.ambient_light.rgb, 1.0);

    if get_distance(pos) <= 0.0 {
        return lighting_color;
    }

#if AVAILABLE_STORAGE_BUFFER_BINDINGS >= 6
    let light_count = arrayLength(&lights);
#else
    let light_count = MAX_LIGHTS;
#endif

    for (var i = 0u; i < light_count; i++) {
        let light = lights[i];
        let dist = distance(light.center, pos);

        if dist < light.radius {
            lighting_color += vec4(light.color.rgb, 1.0) *
                attenuation(light, dist) *
                raymarch(light, pos);
        }
    }

    return lighting_color;
}

fn get_distance(pos: vec2<f32>) -> f32 {
    let uv = ndc_to_uv(position_world_to_ndc(vec3(pos, 0.0)).xy);
    let dist = textureSampleLevel(sdf, sdf_sampler, uv, 0.0).r;
    return dist;
}

fn square(x: f32) -> f32 {
    return x * x;
}

// Attribution: https://lisyarus.github.io/blog/posts/point-light-attenuation.html
fn attenuation(light: PointLight2d, dist: f32) -> f32 {
    let s = dist / light.radius;
    if s > 1.0 {
        return 0.0;
    }
    let s2 = square(s);
    return light.intensity * square(1 - s2) / (1 + light.falloff * s2);
}

// Implementation follows the demo of this article with some enhancements
// https://www.rykap.com/2020/09/23/distance-fields
fn raymarch(light: PointLight2d, ray_origin: vec2<f32>) -> f32 {
    let config = settings.raymarch;
    let max_steps = config.max_steps;
    let sharpness = config.sharpness;
    let jitter = config.jitter;

    let ray_direction = normalize(light.center - ray_origin);
    let stop_at = distance(ray_origin, light.center);

    var ray_progress = 0.0;
    var light_contrib = 1.0;

    for (var i = 0u; i < max_steps; i++) {
        // ray found target
        if (ray_progress > stop_at) {
            // 1.0 next to the light and 0.0 at light.radius away
            let fade_ratio = 1.0 - clamp(stop_at / light.radius, 0.0, 1.0);
            // fade off quadratically instead of linearly
            let distance_factor = pow(fade_ratio, 2.0);

            return light_contrib * distance_factor;
        }

        let dist = get_distance(ray_origin + ray_progress * ray_direction);

        // ray found occluder
        if dist <= 0.0 {
            break;
        }

        light_contrib = min(light_contrib, dist / ray_progress * sharpness);

        ray_progress += dist * (1.0 - jitter) + jitter * fract(dist * 43758.5453);
    }

    return 0.0;
}
