use bevy::{
    ecs::world::World,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_phase::ViewSortedRenderPhases,
        render_resource::{Operations, RenderPassColorAttachment, RenderPassDescriptor},
        renderer::{RenderContext, ViewQuery},
        view::ExtractedView,
    },
};

use crate::render::{Light2dPhase, LightingTextures};

pub fn light2d_render_system(
    world: &World,
    view: ViewQuery<(&ExtractedCamera, &ExtractedView)>,
    light_phases: Res<ViewSortedRenderPhases<Light2dPhase>>,
    mut ctx: RenderContext,
) {
    let view_entity = view.entity();
    let (camera, view) = view.into_inner();

    let Some(light_phase) = light_phases.get(&view.retained_view_entity) else {
        return;
    };

    if light_phase.items.is_empty() {
        return;
    }

    let Some(mut lighting_texture) = world
        .resource::<LightingTextures>()
        .get(&view.retained_view_entity)
        .cloned()
    else {
        return;
    };

    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("light2d_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &lighting_texture.input().default_view,
            resolve_target: None,
            ops: Operations::default(),
            depth_slice: None,
        })],
        ..default()
    });

    if let Some(viewport) = camera.viewport.as_ref() {
        pass.set_camera_viewport(viewport);
    }

    if let Err(err) = light_phase.render(&mut pass, world, view_entity) {
        error!("Error encountered while rendering the lighting phase {err:?}");
    }

    lighting_texture.flip();
}
