#import bevy_render::view::View
#import bevy_lit::{
    functions::{screen_to_world, world_to_sdf_uv},
    types::{AmbientLight2d, PointLight2d},
}

@group(0) @binding(2) var<uniform> ambient_light: AmbientLight2d;
@group(0) @binding(3) var<storage> lights: array<PointLight2d>;
@group(0) @binding(4) var lighting_texture: texture_storage_2d<rgba16float, read_write>;
@group(0) @binding(5) var sdf: texture_2d<f32>;
@group(0) @binding(6) var sdf_sampler: sampler;

@compute @workgroup_size(16, 16, 1)
fn compute(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = screen_to_world(vec2<f32>(global_id.xy));

    var lighting_color = ambient_light.color;

    if get_distance(pos) <= 0.0 {
        textureStore(lighting_texture, global_id.xy, lighting_color);
        return;
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

    textureStore(lighting_texture, global_id.xy, lighting_color);
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
    let uv = world_to_sdf_uv(pos);
    let dist = textureSampleLevel(sdf, sdf_sampler, uv, 0.0).r;
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
