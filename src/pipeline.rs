use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{query::QueryItem, system::lifetimeless::Read},
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_resource::{
            binding_types::{sampler, texture_2d, texture_storage_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            ColorTargetState, ColorWrites, ComputePassDescriptor, ComputePipelineDescriptor,
            Extent3d, FragmentState, MultisampleState, Operations, PipelineCache, PrimitiveState,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
            SamplerBindingType, ShaderStages, SpecializedRenderPipeline, StorageTextureAccess,
            TextureFormat, TextureSampleType,
        },
        renderer::{RenderContext, RenderDevice},
        texture::{BevyDefault, GpuImage},
        view::{ViewTarget, ViewUniform, ViewUniformOffset},
    },
};

use crate::{
    extract::{
        ExtractedAmbientLight2d, ExtractedLightOccluder2d, ExtractedLighting2dSettings,
        ExtractedPointLight2d,
    },
    gpu_bind_groups::{Lighting2dSurfaceBindGroups, PostProcessPipelineId},
    gpu_resources::{LightingArrayBuffer, LightingUniform},
    surfaces::{BLUR_SURFACE, SURFACE},
    utils::workgroup_grid_from_image_size,
};

pub const TYPES_SHADER: Handle<Shader> = Handle::weak_from_u128(76578417911493);
pub const FUNCTIONS_SHADER: Handle<Shader> = Handle::weak_from_u128(87346319816813);
pub const SDF_SHADER: Handle<Shader> = Handle::weak_from_u128(57492774892945);
pub const LIGHTING_SHADER: Handle<Shader> = Handle::weak_from_u128(47320975447604);
pub const BLUR_SHADER: Handle<Shader> = Handle::weak_from_u128(43806754295913);
pub const POST_PROCESS_SHADER: Handle<Shader> = Handle::weak_from_u128(57420546547174);

const WORKGROUP_SIZE: u32 = 16;

#[derive(Debug, Resource)]
pub struct LightingPipelines {
    pub sdf_layout: BindGroupLayout,
    pub sdf_pipeline: CachedComputePipelineId,
    pub lighting_layout: BindGroupLayout,
    pub lighting_pipeline: CachedComputePipelineId,
    pub blur_layout: BindGroupLayout,
    pub blur_pipeline: CachedComputePipelineId,
}

impl FromWorld for LightingPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let sdf_layout = render_device.create_bind_group_layout(
            "sdf_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<ViewUniform>(true),
                    LightingUniform::<ExtractedLighting2dSettings>::binding_layout(),
                    LightingArrayBuffer::<ExtractedLightOccluder2d>::binding_layout(),
                    texture_storage_2d(TextureFormat::Rgba16Float, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        let sdf_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("sdf_pipeline".into()),
            layout: vec![sdf_layout.clone()],
            push_constant_ranges: vec![],
            shader: SDF_SHADER,
            shader_defs: vec![],
            entry_point: "compute".into(),
        });

        let lighting_layout = render_device.create_bind_group_layout(
            "lighting2d_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<ViewUniform>(true),
                    LightingUniform::<ExtractedLighting2dSettings>::binding_layout(),
                    LightingUniform::<ExtractedAmbientLight2d>::binding_layout(),
                    LightingArrayBuffer::<ExtractedPointLight2d>::binding_layout(),
                    texture_storage_2d(TextureFormat::Rgba16Float, StorageTextureAccess::WriteOnly),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let lighting_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("lighting2d_pipeline".into()),
            layout: vec![lighting_layout.clone()],
            push_constant_ranges: vec![],
            shader: LIGHTING_SHADER,
            shader_defs: vec![],
            entry_point: "compute".into(),
        });

        let blur_layout = render_device.create_bind_group_layout(
            "blur_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    LightingUniform::<ExtractedLighting2dSettings>::binding_layout(),
                    texture_storage_2d(TextureFormat::Rgba16Float, StorageTextureAccess::WriteOnly),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let blur_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("blur_pipeline".into()),
            layout: vec![blur_layout.clone()],
            push_constant_ranges: vec![],
            shader: BLUR_SHADER,
            shader_defs: vec![],
            entry_point: "compute".into(),
        });

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

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct PostProcessKey {
    pub hdr: bool,
}

impl SpecializedRenderPipeline for PostProcessPipeline {
    type Key = PostProcessKey;

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
        Read<PostProcessPipelineId>,
    );

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        ctx: &mut RenderContext<'w>,
        (view_target, view_uniform, post_process_pipeline_id): QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        if !world.contains_resource::<Lighting2dSurfaceBindGroups>() {
            return Ok(());
        }

        let pipeline_cache = world.resource::<PipelineCache>();
        let lighting_pipeline = world.resource::<LightingPipelines>();
        let images = world.resource::<RenderAssets<GpuImage>>();

        let (
            Some(sdf_pipeline),
            Some(lighting_pipeline),
            Some(blur_pipeline),
            Some(post_process_pipeline),
            Some(surface),
            Some(blur_surface),
        ) = (
            pipeline_cache.get_compute_pipeline(lighting_pipeline.sdf_pipeline),
            pipeline_cache.get_compute_pipeline(lighting_pipeline.lighting_pipeline),
            pipeline_cache.get_compute_pipeline(lighting_pipeline.blur_pipeline),
            pipeline_cache.get_render_pipeline(post_process_pipeline_id.id),
            images.get(&SURFACE),
            images.get(&BLUR_SURFACE),
        )
        else {
            return Ok(());
        };

        let Lighting2dSurfaceBindGroups {
            lighting: lighting_bind_group,
            sdf: sdf_bind_group,
            blur: blur_bind_group,
        } = world.resource::<Lighting2dSurfaceBindGroups>();

        let mut lighting_pass = ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("lighting_pass"),
                ..default()
            });

        let workgroup_grid = workgroup_grid_from_image_size(surface.size, WORKGROUP_SIZE);

        // SDF
        lighting_pass.set_pipeline(sdf_pipeline);
        lighting_pass.set_bind_group(0, sdf_bind_group, &[view_uniform.offset]);
        lighting_pass.dispatch_workgroups(workgroup_grid.x, workgroup_grid.y, 1);

        // Lighting
        lighting_pass.set_bind_group(0, lighting_bind_group, &[view_uniform.offset]);
        lighting_pass.set_pipeline(lighting_pipeline);
        lighting_pass.dispatch_workgroups(workgroup_grid.x, workgroup_grid.y, 1);

        drop(lighting_pass);

        // Blur
        let should_blur = world.resource::<ExtractedLighting2dSettings>().blur_coc > 0.0;

        if should_blur {
            ctx.command_encoder().copy_texture_to_texture(
                surface.texture.as_image_copy(),
                blur_surface.texture.as_image_copy(),
                Extent3d {
                    width: blur_surface.size.x,
                    height: blur_surface.size.y,
                    ..default()
                },
            );

            let mut blur_pass = ctx
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("blur_pass"),
                    ..default()
                });

            blur_pass.set_pipeline(blur_pipeline);
            blur_pass.set_bind_group(0, blur_bind_group, &[]);
            blur_pass.dispatch_workgroups(workgroup_grid.x, workgroup_grid.y, 1);
        }

        // Post Process
        let post_process = view_target.post_process_write();

        let post_process_bind_group = ctx.render_device().create_bind_group(
            "post_process_bind_group",
            &world.resource::<PostProcessPipeline>().layout,
            &BindGroupEntries::sequential((
                post_process.source,
                if should_blur {
                    &blur_surface.texture_view
                } else {
                    &surface.texture_view
                },
                if should_blur {
                    &blur_surface.sampler
                } else {
                    &surface.sampler
                },
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
