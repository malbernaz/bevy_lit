use bevy::{
    ecs::{query::QueryItem, system::lifetimeless::Read, world::World},
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_phase::{SortedRenderPhase, ViewSortedRenderPhases},
        render_resource::{
            BindGroupEntries, Operations, PipelineCache, RenderPassColorAttachment,
            RenderPassDescriptor, SamplerDescriptor, UniformBuffer,
        },
        renderer::{RenderContext, RenderQueue},
        view::{ExtractedView, ViewTarget},
    },
};

use crate::{
    render::{FlipTexture, VoronoiPhase, VoronoiTextures},
    voronoi::FloodPipeline, wrappers::UVec2Uniform,
};

pub fn run_mask_pass<'w>(
    world: &'w World,
    render_context: &mut RenderContext<'w>,
    phase: &SortedRenderPhase<VoronoiPhase>,
    view_entity: &Entity,
    voronoi_texture: &mut FlipTexture,
    camera: &ExtractedCamera,
) {
    let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("mask_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &voronoi_texture.output().default_view,
            resolve_target: None,
            ops: Operations::default(),
            depth_slice: None,
        })],
        ..default()
    });

    if let Some(viewport) = camera.viewport.as_ref() {
        pass.set_camera_viewport(viewport);
    }

    if let Err(err) = phase.render(&mut pass, world, *view_entity) {
        error!("Error encountered while rendering the voronoi mask phase {err:?}");
    }

    voronoi_texture.flip();
}

pub fn run_flood_seed_pass<'w>(
    world: &'w World,
    render_context: &mut RenderContext<'w>,
    camera: &ExtractedCamera,
    voronoi_texture: &mut FlipTexture,
) {
    let flood_pipeline = world.resource::<FloodPipeline>();

    let Some(pipeline) = world
        .resource::<PipelineCache>()
        .get_render_pipeline(flood_pipeline.seed_pipeline)
    else {
        return;
    };

    let sampler = render_context
        .render_device()
        .create_sampler(&SamplerDescriptor::default());

    let bind_group = render_context.render_device().create_bind_group(
        "flood_seed_bind_group",
        &flood_pipeline.seed_layout,
        &BindGroupEntries::sequential((&voronoi_texture.input().default_view, &sampler)),
    );

    let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("flood_seed_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &voronoi_texture.output().default_view,
            resolve_target: None,
            ops: Operations::default(),
            depth_slice: None,
        })],
        ..default()
    });

    if let Some(viewport) = camera.viewport.as_ref() {
        pass.set_camera_viewport(viewport);
    }

    pass.set_render_pipeline(pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.draw(0..3, 0..1);

    voronoi_texture.flip();
}

pub fn run_flood_pass<'w>(
    world: &'w World,
    render_context: &mut RenderContext<'w>,
    camera: &ExtractedCamera,
    voronoi_texture: &mut FlipTexture,
    step: UVec2Uniform,
) {
    let flood_pipeline = world.resource::<FloodPipeline>();

    let mut step = UniformBuffer::from(step);

    step.write_buffer(
        render_context.render_device(),
        world.resource::<RenderQueue>(),
    );

    let (Some(pipeline), Some(step)) = (
        world
            .resource::<PipelineCache>()
            .get_render_pipeline(flood_pipeline.pipeline),
        step.binding(),
    ) else {
        return;
    };

    let sampler = render_context
        .render_device()
        .create_sampler(&SamplerDescriptor::default());

    let bind_group = render_context.render_device().create_bind_group(
        "flood_bind_group",
        &flood_pipeline.layout,
        &BindGroupEntries::sequential((&voronoi_texture.input().default_view, &sampler, step)),
    );

    let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("flood_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &voronoi_texture.output().default_view,
            resolve_target: None,
            ops: Operations::default(),
            depth_slice: None,
        })],
        ..default()
    });

    if let Some(viewport) = camera.viewport.as_ref() {
        pass.set_camera_viewport(viewport);
    }

    pass.set_render_pipeline(pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.draw(0..3, 0..1);

    voronoi_texture.flip();
}

#[derive(Default)]
pub struct VoronoiDrawNode;
impl ViewNode for VoronoiDrawNode {
    type ViewQuery = (Read<ExtractedCamera>, Read<ExtractedView>, Read<ViewTarget>);

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (camera, view, target): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.view_entity();

        let Some(mask_phase) = world
            .resource::<ViewSortedRenderPhases<VoronoiPhase>>()
            .get(&view.retained_view_entity)
        else {
            return Ok(());
        };

        if mask_phase.items.is_empty() {
            return Ok(());
        }

        let mut voronoi_texture = world
            .resource::<VoronoiTextures>()
            .get(&view.retained_view_entity)
            .expect(&format!(
                "Expected the voronoi texture for {:?} exist",
                view.retained_view_entity.main_entity.id()
            ))
            .clone();

        run_mask_pass(
            world,
            render_context,
            mask_phase,
            &view_entity,
            &mut voronoi_texture,
            camera,
        );

        run_flood_seed_pass(world, render_context, camera, &mut voronoi_texture);

        let width = target.main_texture().width();
        let height = target.main_texture().height();
        let max_dim = width.max(height);
        let mut step = max_dim / 2;

        while step >= 1 {
            let x_step = (step * width) / max_dim;
            let y_step = (step * height) / max_dim;

            run_flood_pass(
                world,
                render_context,
                camera,
                &mut voronoi_texture,
                UVec2Uniform::new(x_step.max(1), y_step.max(1)),
            );

            step /= 2;
        }

        // Addicional pass with step = 1 to improve accuracy
        run_flood_pass(
            world,
            render_context,
            camera,
            &mut voronoi_texture,
            UVec2Uniform::new(1, 1),
        );

        Ok(())
    }
}
