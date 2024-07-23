#import bevy_render::view::View
#import bevy_lit::{
    functions::{screen_to_world},
    types::{LightingSettings, LightOccluder2d},
}

@group(0) @binding(2) var<storage> occluders: array<LightOccluder2d>;
@group(0) @binding(3) var texture: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn compute(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = screen_to_world(vec2<f32>(global_id.xy));

    var sdf = occluder_sd(pos.xy, occluders[0]);

    for (var i = 1u; i < arrayLength(&occluders); i++) {
        sdf = min(sdf, occluder_sd(pos.xy, occluders[i]));
    }

    textureStore(texture, global_id.xy, vec4(sdf, 0.0, 0.0, 1.0));
}

fn occluder_sd(p: vec2f, occluder: LightOccluder2d) -> f32 {
  let local_pos = occluder.center - p;
  let d = abs(local_pos) - occluder.half_size;
  return length(max(d, vec2f(0.))) + min(max(d.x, d.y), 0.);
}
