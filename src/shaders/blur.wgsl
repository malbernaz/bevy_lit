#import bevy_lit::types::LightingSettings

@group(0) @binding(0) var<uniform> settings: LightingSettings;
@group(0) @binding(1) var texture: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var texture_copy: texture_2d<f32>;
@group(0) @binding(3) var texture_sampler: sampler;

@compute @workgroup_size(16, 16, 1)
fn compute(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = gaussian_blur(vec4<f32>(vec2<f32>(id.xy), 0.0, 1.0), settings.coc, vec2(1.0, 0.0));
    let y = gaussian_blur(vec4<f32>(vec2<f32>(id.xy), 0.0, 1.0), settings.coc, vec2(0.0, 1.0));
    textureStore(texture, id.xy, mix(x, y, 0.5));
}

// ATTRIBUTION: The code for this function was originally
// contributed to bevy under the MIT or Apache 2 licenses.
fn gaussian_blur(frag_coord: vec4<f32>, coc: f32, frag_offset: vec2<f32>) -> vec4<f32> {
    let sigma = coc * 0.25;
    let support = i32(ceil(sigma * 1.5));
    let uv = frag_coord.xy / vec2<f32>(textureDimensions(texture_copy));
    let offset = frag_offset / vec2<f32>(textureDimensions(texture_copy));
    let exp_factor = -1.0 / (2.0 * sigma * sigma);

    var sum = textureSampleLevel(texture_copy, texture_sampler, uv, 0.0).rgb;
    var weight_sum = 1.0;

    for (var i = 1; i <= support; i += 2) {
        let w0 = exp(exp_factor * f32(i) * f32(i));
        let w1 = exp(exp_factor * f32(i + 1) * f32(i + 1));
        let uv_offset = offset * (f32(i) + w1 / (w0 + w1));
        let weight = w0 + w1;

        sum += (
            textureSampleLevel(texture_copy, texture_sampler, uv + uv_offset, 0.0).rgb +
            textureSampleLevel(texture_copy, texture_sampler, uv - uv_offset, 0.0).rgb
        ) * weight;

        weight_sum += weight * 2.0;
    }

    return vec4(sum / weight_sum, 1.0);
}

