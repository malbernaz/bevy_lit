#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_lit::wrappers_types::UVec2Uniform

@group(0) @binding(0) var seed_texture: texture_2d<f32>;
@group(0) @binding(1) var sampler_obj: sampler;
@group(0) @binding(2) var<uniform> step: UVec2Uniform;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let original_seed = textureSample(seed_texture, sampler_obj, in.uv);

    if original_seed.z == 1. {
        return original_seed;
    }

    let dims = vec2<f32>(textureDimensions(seed_texture));

    var current_seed = original_seed.xy;
    var current_dist = 9999999999.0;

    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let neighbour_coords = in.position.xy + vec2<f32>(vec2<i32>(x, y) * vec2<i32>(step.inner));
            let neighbour_seed = textureSampleLevel(seed_texture, sampler_obj, neighbour_coords / dims, 0.0).xy;
            let neighbour_dist = length(in.position.xy - neighbour_seed);

            if neighbour_seed.x >= 0. && neighbour_seed.y >= 0. && neighbour_dist < current_dist {
                current_seed = neighbour_seed;
                current_dist = neighbour_dist;
            }
        }
    }

    return vec4<f32>(current_seed, 0.0, original_seed.w);
}
