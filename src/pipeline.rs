use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{query::QueryItem, system::lifetimeless::Read},
    prelude::*,
    render::{
        extract_component::DynamicUniformIndex,
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FragmentState, GpuArrayBuffer, LoadOp, MultisampleState,
            Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor,
            ShaderStages, SpecializedRenderPipeline, StoreOp, TextureFormat, TextureSampleType,
        },
        renderer::{RenderContext, RenderDevice},
        texture::BevyDefault,
        view::{ViewTarget, ViewUniform, ViewUniformOffset},
    },
};

use crate::{
    extract::{ExtractedLightOccluder2d, ExtractedLighting2dSettings, ExtractedPointLight2d},
    prepare::{
        Lighting2dAuxiliaryTextures, Lighting2dPostProcessPipelineId, Lighting2dSurfaceBindGroups,
    },
};

pub const TYPES_SHADER: Handle<Shader> = Handle::weak_from_u128(76578417911493);
pub const VIEW_TRANSFORMATIONS_SHADER: Handle<Shader> = Handle::weak_from_u128(43290875047924);
pub const SDF_SHADER: Handle<Shader> = Handle::weak_from_u128(57492774892945);
pub const LIGHTING_SHADER: Handle<Shader> = Handle::weak_from_u128(47320975447604);
pub const BLUR_SHADER: Handle<Shader> = Handle::weak_from_u128(43806754295913);
pub const POST_PROCESS_SHADER: Handle<Shader> = Handle::weak_from_u128(57420546547174);

fn create_pipeline_descriptor(
    pipeline_cache: &PipelineCache,
    label: &'static str,
    layout: &BindGroupLayout,
    shader: Handle<Shader>,
) -> CachedRenderPipelineId {
    pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some(label.into()),
        layout: vec![layout.clone()],
        vertex: fullscreen_shader_vertex_state(),
        fragment: Some(FragmentState {
            shader,
            shader_defs: vec![],
            entry_point: "fragment".into(),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba16Float,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        push_constant_ranges: vec![],
    })
}

#[derive(Resource)]
pub struct Lighting2dPrepassPipelines {
    pub sdf_layout: BindGroupLayout,
    pub sdf_pipeline: CachedRenderPipelineId,
    pub lighting_layout: BindGroupLayout,
    pub lighting_pipeline: CachedRenderPipelineId,
    pub blur_layout: BindGroupLayout,
    pub blur_pipeline: CachedRenderPipelineId,
}

impl FromWorld for Lighting2dPrepassPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let sdf_layout = render_device.create_bind_group_layout(
            "sdf_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<ViewUniform>(true),
                    GpuArrayBuffer::<ExtractedLightOccluder2d>::binding_layout(render_device),
                ),
            ),
        );

        let sdf_pipeline =
            create_pipeline_descriptor(pipeline_cache, "sdf_pipeline", &sdf_layout, SDF_SHADER);

        let lighting_layout = render_device.create_bind_group_layout(
            "lighting_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<ExtractedLighting2dSettings>(true),
                    GpuArrayBuffer::<ExtractedPointLight2d>::binding_layout(render_device),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let lighting_pipeline = create_pipeline_descriptor(
            pipeline_cache,
            "lighting_pipeline",
            &lighting_layout,
            LIGHTING_SHADER,
        );

        let blur_layout = render_device.create_bind_group_layout(
            "blur_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<ViewUniform>(true),
                    uniform_buffer::<ExtractedLighting2dSettings>(true),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let blur_pipeline =
            create_pipeline_descriptor(pipeline_cache, "blur_pipeline", &blur_layout, BLUR_SHADER);

        Self {
            sdf_layout,
            sdf_pipeline,
            lighting_layout,
            lighting_pipeline,
            blur_layout,
            blur_pipeline,
        }
    }
}

#[derive(Resource)]
pub struct PostProcessPipeline {
    pub layout: BindGroupLayout,
}

impl FromWorld for PostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        Self {
            layout: render_device.create_bind_group_layout(
                "post_process_bind_group_layout",
                &BindGroupLayoutEntries::sequential(
                    ShaderStages::FRAGMENT,
                    (
                        texture_2d(TextureSampleType::Float { filterable: true }),
                        texture_2d(TextureSampleType::Float { filterable: true }),
                        sampler(SamplerBindingType::Filtering),
                    ),
                ),
            ),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct Lighting2dPipelineKey {
    pub hdr: bool,
}

impl SpecializedRenderPipeline for PostProcessPipeline {
    type Key = Lighting2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("post_process_pipeline".into()),
            layout: vec![self.layout.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: POST_PROCESS_SHADER,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: if key.hdr {
                        ViewTarget::TEXTURE_FORMAT_HDR
                    } else {
                        TextureFormat::bevy_default()
                    },
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct LightingLabel;

#[derive(Default)]
pub struct LightingNode;

impl ViewNode for LightingNode {
    type ViewQuery = (
        Read<ViewTarget>,
        Read<ViewUniformOffset>,
        Read<Lighting2dPostProcessPipelineId>,
        Read<Lighting2dAuxiliaryTextures>,
        Read<Lighting2dSurfaceBindGroups>,
        Read<DynamicUniformIndex<ExtractedLighting2dSettings>>,
    );

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        ctx: &mut RenderContext<'w>,
        (
            view_target,
            view_uniform,
            post_process_pipeline_id,
            aux_textures,
            bind_groups,
            settings_index,
        ): QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let prepass_pipelines = world.resource::<Lighting2dPrepassPipelines>();

        let (
            Some(sdf_pipeline),
            Some(lighting_pipeline),
            Some(blur_pipeline),
            Some(post_process_pipeline),
        ) = (
            pipeline_cache.get_render_pipeline(prepass_pipelines.sdf_pipeline),
            pipeline_cache.get_render_pipeline(prepass_pipelines.lighting_pipeline),
            pipeline_cache.get_render_pipeline(prepass_pipelines.blur_pipeline),
            pipeline_cache.get_render_pipeline(post_process_pipeline_id.0),
        )
        else {
            return Ok(());
        };

        let storage_buffer_support = ctx
            .render_device()
            .limits()
            .max_storage_buffers_per_shader_stage
            > 0;

        // SDF
        let mut sdf_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("sdf_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &aux_textures.sdf.default_view,
                resolve_target: None,
                ops: Operations::default(),
            })],
            ..default()
        });

        let mut dynamic_offset = vec![view_uniform.offset];
        if !storage_buffer_support {
            dynamic_offset.push(0);
        }

        sdf_pass.set_render_pipeline(sdf_pipeline);
        sdf_pass.set_bind_group(0, &bind_groups.sdf, &dynamic_offset[..]);
        sdf_pass.draw(0..3, 0..1);

        drop(sdf_pass);

        // Lighting
        let mut lighting_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("lighting_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &aux_textures.lighting.default_view,
                resolve_target: None,
                ops: Operations::default(),
            })],
            ..default()
        });

        let mut dynamic_offset = vec![view_uniform.offset, settings_index.index()];
        if !storage_buffer_support {
            dynamic_offset.push(0);
        }

        lighting_pass.set_render_pipeline(lighting_pipeline);
        lighting_pass.set_bind_group(0, &bind_groups.lighting, &dynamic_offset[..]);
        lighting_pass.draw(0..3, 0..1);

        drop(lighting_pass);

        // Blur
        if let Some(blur_texture) = &aux_textures.blur {
            let mut blur_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("blur_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &blur_texture.default_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                ..default()
            });

            blur_pass.set_bind_group(
                0,
                &bind_groups.blur,
                &[view_uniform.offset, settings_index.index()],
            );
            blur_pass.set_render_pipeline(blur_pipeline);
            blur_pass.draw(0..3, 0..1);
        }

        // Post Process
        let post_process = view_target.post_process_write();

        let sampler = ctx
            .render_device()
            .create_sampler(&SamplerDescriptor::default());

        let post_process_bind_group = ctx.render_device().create_bind_group(
            "post_process_bind_group",
            &world.resource::<PostProcessPipeline>().layout,
            &BindGroupEntries::sequential((
                post_process.source,
                if let Some(blur_texture) = &aux_textures.blur {
                    &blur_texture.default_view
                } else {
                    &aux_textures.lighting.default_view
                },
                &sampler,
            )),
        );

        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            ..default()
        });

        pass.set_bind_group(0, &post_process_bind_group, &[]);
        pass.set_render_pipeline(post_process_pipeline);
        pass.draw(0..3, 0..1);

        Ok(())
    }
}
