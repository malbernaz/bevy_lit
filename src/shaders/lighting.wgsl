#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_lit::types::{AmbientLight2d, PointLight2d},
#import bevy_pbr::view_transformations::{
    frag_coord_to_ndc,
    position_ndc_to_world,
    position_world_to_ndc,
    ndc_to_uv,
}

@group(0) @binding(1) var<uniform> ambient_light: AmbientLight2d;
@group(0) @binding(2) var<storage> lights: array<PointLight2d>;
@group(0) @binding(3) var sdf: texture_2d<f32>;
@group(0) @binding(4) var sdf_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pos = position_ndc_to_world(frag_coord_to_ndc(in.position)).xy;

    var lighting_color = ambient_light.color;

    if get_distance(pos) <= 0.0 {
        return lighting_color;
    }

    for (var i = 0u; i < arrayLength(&lights); i++) {
        let light = lights[i];
        let dist = distance(light.center, pos);

        if dist < light.radius {
            let raymarch = raymarch(pos, light.center);

            if raymarch > 0.0 {
                lighting_color += light.color * attenuation(light, dist);
            }
        }
    }

    return lighting_color;
}

fn square(x: f32) -> f32 {
    return x * x;
}

fn attenuation(light: PointLight2d, dist: f32) -> f32 {
    let s = dist / light.radius;
    if s > 1.0 {
        return 0.0;
    }
    let s2 = square(s);
    return light.intensity * square(1 - s2) / (1 + light.falloff * s2);
}

fn get_distance(pos: vec2<f32>) -> f32 {
    let uv = ndc_to_uv(position_world_to_ndc(vec3(pos, 0.0)).xy);
    let dist = textureSample(sdf, sdf_sampler, uv).r;
    return dist;
}

fn distance_squared(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let c = a - b;
    return dot(c, c);
}

fn raymarch(ray_origin: vec2<f32>, ray_target: vec2<f32>) -> f32 {
    let ray_direction = normalize(ray_target - ray_origin);
    let stop_at = distance_squared(ray_origin, ray_target);

    var ray_progress: f32 = 0.0;
    var pos = vec2<f32>(0.0);

    for (var i = 0; i < 32; i++) {
        pos = ray_origin + ray_progress * ray_direction;

        if (ray_progress * ray_progress >= stop_at) {
            // ray found target
            return 1.0;
        }

        let dist = get_distance(pos);

        if dist <= 0.0 {
            break;
        }

        ray_progress += dist;
    }

    // ray found occluder
    return 0.0;
}
