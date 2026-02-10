#import bevy_lit::{
    view_transformations::frag_to_world,
    light2d_common::{VertexOutput, settings, view, raymarch}
}

@group(1) @binding(0) var gradient_map: texture_1d<f32>;
@group(1) @binding(1) var<uniform> radius: vec4<f32>;
@group(1) @binding(2) var<uniform> color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = frag_to_world(in.clip_position / settings.scale, view).xy;
    let light_center = in.translation_rotation.xy;
    let light_dist = distance(pos, light_center);

    let t = 1.0 - (light_dist / radius.x);
    let n_levels = f32(textureDimensions(gradient_map));
    let idx = u32(ceil(t * (n_levels - 1.0)));

    let gradient_contrib = textureLoad(gradient_map, idx, 0).r;
    let shadow_contrib = raymarch(pos, light_center);

    return vec4<f32>(color.rgb * gradient_contrib * shadow_contrib, 1.0);
}
