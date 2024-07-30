#define_import_path bevy_lit::functions

#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

fn screen_to_ndc(pos: vec2<f32>) -> vec2<f32> {
    let screen_size = view.viewport.zw;
    let screen_size_inv = vec2<f32>(1.0 / screen_size.x, 1.0 / screen_size.y);
    let ndc = vec2<f32>(pos.x, screen_size.y - pos.y);
    return (ndc * screen_size_inv) * 2.0 - 1.0;
}

fn screen_to_world(pos: vec2<f32>) -> vec2<f32> {
    return (view.world_from_clip * vec4<f32>(screen_to_ndc(pos), 0.0, 1.0)).xy;
}

fn world_to_ndc(pos: vec2<f32>) -> vec2<f32> {
    return (view.clip_from_world * vec4<f32>(pos, 0.0, 1.0)).xy;
}

fn world_to_sdf_uv(pos: vec2<f32>) -> vec2<f32> {
    let ndc = world_to_ndc(pos);
    let uv = (ndc + 1.0) * 0.5;
    return vec2<f32>(uv.x, 1.0 - uv.y);
}
