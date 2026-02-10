#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_lit::{settings_types::Lighting2dSettings, wrappers_types::IVec2Uniform}

@group(0) @binding(0) var<uniform> settings: Lighting2dSettings;
@group(0) @binding(1) var<uniform> direction: IVec2Uniform;
@group(0) @binding(2) var texture: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return gaussian_blur(in.position.xy, direction.inner, settings.blur);
}

fn gaussian_weight(x: f32, sigma: f32) -> f32 {
    return exp(-0.5 * (x * x) / (sigma * sigma));
}

fn gaussian_blur(frag_pos: vec2<f32>, direction: vec2<i32>, radius: i32) -> vec4<f32> {
    let sigma = f32(radius) * 0.25;
    let texel_pos = vec2<i32>(frag_pos);
    let tex_size = vec2<i32>(textureDimensions(texture));

    var color = vec4<f32>(0.0);
    var total_weight: f32 = 0.0;

    for (var i = -radius; i <= radius; i++) {
        let sample_pos = clamp(texel_pos + direction * i, vec2<i32>(0), tex_size - vec2<i32>(1));
        let w = gaussian_weight(f32(i), sigma);
        color += textureLoad(texture, sample_pos, 0) * w;
        total_weight += w;
    }

    return color / total_weight;
}
