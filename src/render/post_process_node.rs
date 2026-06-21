use bevy::{
    ecs::{query::QueryItem, system::lifetimeless::Read},
    prelude::*,
    render::{
        camera::ExtractedCamera,
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_resource::{
            BindGroupEntries, CachedRenderPipelineId, Operations, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor, SamplerDescriptor, UniformBuffer,
        },
        renderer::{RenderContext, RenderQueue},
        view::{ExtractedView, ViewTarget, ViewUniformOffset, ViewUniforms},
    },
};

use crate::{
    post_process::render::{
        ExtractedLighting2dSettings, Lighting2dCompositePipeline, Lighting2dCompositePipelineId,
        Lighting2dPostProcessPipelines,
    },
    render::{FlipTexture, LightingTextures, VoronoiTextures},
    settings::PenetrationSettings,
};

pub fn run_penetration_pass<'w>(
    world: &'w World,
    render_context: &mut RenderContext<'w>,
    camera: &ExtractedCamera,
    lighting_texture: &mut FlipTexture,
    voronoi_texture: &FlipTexture,
    view_uniform_offset: u32,
    settings_uniform_offset: u32,
) {
    let post_process_pipelines = world.resource::<Lighting2dPostProcessPipelines>();
    let pipeline_cache = world.resource::<PipelineCache>();

    let (Some(pipeline), Some(view_uniforms), Some(lighting_settings_uniforms)) = (
        pipeline_cache.get_render_pipeline(post_process_pipelines.penetration_pipeline),
        world.resource::<ViewUniforms>().uniforms.binding(),
        world
            .resource::<ComponentUniforms<ExtractedLighting2dSettings>>()
            .binding(),
    ) else {
        return;
    };

    let sampler = render_context
        .render_device()
        .create_sampler(&SamplerDescriptor::default());

    let bind_group = render_context.render_device().create_bind_group(
        "penetration_bind_group",
        &pipeline_cache.get_bind_group_layout(&post_process_pipelines.penetration_layout),
        &BindGroupEntries::sequential((
            view_uniforms,
            lighting_settings_uniforms,
            &lighting_texture.input().default_view,
            &voronoi_texture.input().default_view,
            &sampler,
        )),
    );

    let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("penetration_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &lighting_texture.output().default_view,
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
    pass.set_bind_group(
        0,
        &bind_group,
        &[view_uniform_offset, settings_uniform_offset],
    );
    pass.draw(0..3, 0..1);

    lighting_texture.flip()
}

pub fn run_blur_pass<'w>(
    world: &'w World,
    render_context: &mut RenderContext<'w>,
    lighting_texture: &mut FlipTexture,
    settings_uniform_offset: u32,
    direction: IVec2,
) {
    let post_process_pipelines = world.resource::<Lighting2dPostProcessPipelines>();
    let pipeline_cache = world.resource::<PipelineCache>();

    let mut direction = UniformBuffer::from(direction);
    direction.write_buffer(
        render_context.render_device(),
        world.resource::<RenderQueue>(),
    );
    let (Some(pipeline), Some(lighting_settings_uniforms), Some(direction)) = (
        pipeline_cache.get_render_pipeline(post_process_pipelines.blur_pipeline),
        world
            .resource::<ComponentUniforms<ExtractedLighting2dSettings>>()
            .binding(),
        direction.binding(),
    ) else {
        return;
    };

    let bind_group = render_context.render_device().create_bind_group(
        "blur_bind_group",
        &pipeline_cache.get_bind_group_layout(&post_process_pipelines.blur_layout),
        &BindGroupEntries::sequential((
            lighting_settings_uniforms,
            direction,
            &lighting_texture.input().default_view,
        )),
    );

    let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("blur_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &lighting_texture.output().default_view,
            resolve_target: None,
            ops: Operations::default(),
            depth_slice: None,
        })],
        ..default()
    });

    pass.set_render_pipeline(pipeline);
    pass.set_bind_group(0, &bind_group, &[settings_uniform_offset]);
    pass.draw(0..3, 0..1);

    lighting_texture.flip();
}

pub fn run_composite_pass<'w>(
    world: &'w World,
    render_context: &mut RenderContext<'w>,
    lighting_texture: &mut FlipTexture,
    view_target: &ViewTarget,
    pipeline_id: CachedRenderPipelineId,
    settings_uniform_offset: u32,
) {
    let pipeline_cache = world.resource::<PipelineCache>();

    let (Some(pipeline), Some(lighting_settings_uniforms)) = (
        pipeline_cache.get_render_pipeline(pipeline_id),
        world
            .resource::<ComponentUniforms<ExtractedLighting2dSettings>>()
            .binding(),
    ) else {
        return;
    };

    let post_process = view_target.post_process_write();
    let sampler = render_context
        .render_device()
        .create_sampler(&SamplerDescriptor::default());

    let bind_group = render_context.render_device().create_bind_group(
        "composite_bind_group",
        &pipeline_cache
            .get_bind_group_layout(&world.resource::<Lighting2dCompositePipeline>().layout),
        &BindGroupEntries::sequential((
            lighting_settings_uniforms,
            post_process.source,
            &lighting_texture.input().default_view,
            &sampler,
        )),
    );

    let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("composite_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: post_process.destination,
            resolve_target: None,
            ops: Operations::default(),
            depth_slice: None,
        })],
        ..default()
    });

    pass.set_render_pipeline(pipeline);
    pass.set_bind_group(0, &bind_group, &[settings_uniform_offset]);
    pass.draw(0..3, 0..1);
}

#[derive(Default)]
pub struct Light2dPostProcessDrawNode;
impl ViewNode for Light2dPostProcessDrawNode {
    type ViewQuery = (
        Read<ExtractedView>,
        Read<ViewTarget>,
        Read<ExtractedCamera>,
        Read<ViewUniformOffset>,
        Read<Lighting2dCompositePipelineId>,
        Read<DynamicUniformIndex<ExtractedLighting2dSettings>>,
        Read<ExtractedLighting2dSettings>,
    );

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (
            view,
            view_target,
            camera,
            view_uniform_offset,
            composite_pipeline_id,
            settings_uniform_index,
            lighting_settings,
        ): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> std::result::Result<(), NodeRunError> {
        let mut lighting_texture = world
            .resource::<LightingTextures>()
            .get(&view.retained_view_entity)
            .expect(&format!(
                "Expected the lighting texture for view {:?} to exist",
                view.retained_view_entity.main_entity.id()
            ))
            .clone();
        let voronoi_texture = world
            .resource::<VoronoiTextures>()
            .get(&view.retained_view_entity)
            .expect(&format!(
                "Expected the voronoi texture for view {:?} to exist",
                view.retained_view_entity.main_entity.id()
            ))
            .clone();

        if should_run_penetration_pass(&lighting_settings.penetration) {
            run_penetration_pass(
                world,
                render_context,
                camera,
                &mut lighting_texture,
                &voronoi_texture,
                view_uniform_offset.offset,
                settings_uniform_index.index(),
            );
        }

        if lighting_settings.blur > 0 {
            run_blur_pass(
                world,
                render_context,
                &mut lighting_texture,
                settings_uniform_index.index(),
                IVec2::new(1, 0),
            );
            run_blur_pass(
                world,
                render_context,
                &mut lighting_texture,
                settings_uniform_index.index(),
                IVec2::new(0, 1),
            );
        }

        run_composite_pass(
            world,
            render_context,
            &mut lighting_texture,
            view_target,
            composite_pipeline_id.0,
            settings_uniform_index.index(),
        );

        Ok(())
    }
}

fn should_run_penetration_pass(penetration: &PenetrationSettings) -> bool {
    penetration.max > 0.0
        && penetration.intensity > 0.0
        && penetration.sample_directions > 0
        && penetration.sample_steps > 0
}
