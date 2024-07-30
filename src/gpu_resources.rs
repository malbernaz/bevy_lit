use bevy::{
    prelude::*,
    render::{
        render_resource::{
            binding_types::{storage_buffer_read_only, uniform_buffer},
            encase::internal::WriteInto,
            BindGroup, BindGroupEntries, BindGroupLayoutEntryBuilder, BindingResource,
            CachedRenderPipelineId, GpuArrayBufferable, PipelineCache, SamplerDescriptor,
            ShaderType, SpecializedRenderPipelines, StorageBuffer, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, TextureCache},
        view::{ExtractedView, ViewTarget, ViewUniforms},
    },
};

use crate::{
    extract::{
        ExtractedAmbientLight2d, ExtractedLightOccluder2d, ExtractedLighting2dSettings,
        ExtractedPointLight2d,
    },
    pipeline::{Lighting2dPipelineKey, Lighting2dPrepassPipelines, PostProcessPipeline},
};

#[derive(Resource)]
pub struct LightingUniform<T: ShaderType> {
    value: UniformBuffer<T>,
}

impl<T: ShaderType + WriteInto> LightingUniform<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UniformBuffer::from(value),
        }
    }

    pub fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.value.write_buffer(render_device, render_queue);
    }

    pub fn binding(&self) -> Option<BindingResource> {
        self.value.binding()
    }

    pub fn binding_layout() -> BindGroupLayoutEntryBuilder {
        uniform_buffer::<T>(false)
    }
}

#[derive(Resource)]
pub struct LightingArrayBuffer<T: GpuArrayBufferable> {
    value: StorageBuffer<Vec<T>>,
}

impl<T: GpuArrayBufferable> LightingArrayBuffer<T> {
    pub fn new(value: Vec<T>) -> Self {
        Self {
            value: StorageBuffer::from(value),
        }
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.value.write_buffer(device, queue);
    }

    pub fn binding(&self) -> Option<BindingResource> {
        self.value.binding()
    }

    pub fn binding_layout() -> BindGroupLayoutEntryBuilder {
        storage_buffer_read_only::<T>(false)
    }
}

pub fn prepare_gpu_resources(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    ambient_light: Res<ExtractedAmbientLight2d>,
    lighting_settings: Res<ExtractedLighting2dSettings>,
    point_lights_query: Query<&ExtractedPointLight2d>,
    light_occluders_query: Query<&ExtractedLightOccluder2d>,
) {
    let mut gpu_ambient_light = LightingUniform::new(ambient_light.clone());
    gpu_ambient_light.write_buffer(&render_device, &render_queue);
    commands.insert_resource(gpu_ambient_light);

    let mut gpu_lighting_settings = LightingUniform::new(lighting_settings.clone());
    gpu_lighting_settings.write_buffer(&render_device, &render_queue);
    commands.insert_resource(gpu_lighting_settings);

    let mut gpu_point_lights = LightingArrayBuffer::<ExtractedPointLight2d>::new(
        point_lights_query.iter().cloned().collect::<Vec<_>>(),
    );
    gpu_point_lights.write_buffer(&render_device, &render_queue);
    commands.insert_resource(gpu_point_lights);

    let mut gpu_light_occluders = LightingArrayBuffer::<ExtractedLightOccluder2d>::new(
        light_occluders_query.iter().cloned().collect::<Vec<_>>(),
    );
    gpu_light_occluders.write_buffer(&render_device, &render_queue);
    commands.insert_resource(gpu_light_occluders);
}

#[derive(Component)]
pub struct Lighting2dSurfaceBindGroups {
    pub sdf: BindGroup,
    pub lighting: BindGroup,
    pub blur_x: BindGroup,
    pub blur_y: BindGroup,
}

pub fn prepare_lighting_bind_groups(
    mut commands: Commands,
    prepass_pipelines: Res<Lighting2dPrepassPipelines>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
    ambient_light: Res<LightingUniform<ExtractedAmbientLight2d>>,
    light_settings: Res<LightingUniform<ExtractedLighting2dSettings>>,
    point_lights: Res<LightingArrayBuffer<ExtractedPointLight2d>>,
    light_occluders: Res<LightingArrayBuffer<ExtractedLightOccluder2d>>,
    views_query: Query<(Entity, &Lighting2dAuxiliaryTextures)>,
) {
    let (
        Some(view_uniform),
        Some(ambient_light),
        Some(lighting_settings),
        Some(light_occluders),
        Some(point_lights),
    ) = (
        view_uniforms.uniforms.binding(),
        ambient_light.binding(),
        light_settings.binding(),
        light_occluders.binding(),
        point_lights.binding(),
    )
    else {
        return;
    };

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    for (entity, aux_textures) in &views_query {
        commands.entity(entity).insert(Lighting2dSurfaceBindGroups {
            sdf: render_device.create_bind_group(
                "sdf_bind_group",
                &prepass_pipelines.sdf_layout,
                &BindGroupEntries::sequential((view_uniform.clone(), light_occluders.clone())),
            ),
            lighting: render_device.create_bind_group(
                "lighting2d_bind_group",
                &prepass_pipelines.lighting_layout,
                &BindGroupEntries::sequential((
                    view_uniform.clone(),
                    ambient_light.clone(),
                    point_lights.clone(),
                    &aux_textures.sdf.default_view,
                    &sampler,
                )),
            ),
            blur_x: render_device.create_bind_group(
                "blur_x_bind_group",
                &prepass_pipelines.blur_layout,
                &BindGroupEntries::sequential((
                    view_uniform.clone(),
                    lighting_settings.clone(),
                    &aux_textures.lighting.default_view,
                    &sampler,
                )),
            ),
            blur_y: render_device.create_bind_group(
                "blur_y_bind_group",
                &prepass_pipelines.blur_layout,
                &BindGroupEntries::sequential((
                    view_uniform.clone(),
                    lighting_settings.clone(),
                    &aux_textures.blur_x.default_view,
                    &sampler,
                )),
            ),
        });
    }
}

#[derive(Component)]
pub struct Lighting2dPostProcessPipelineId(pub CachedRenderPipelineId);

pub fn prepare_post_process_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut post_process_pipelines: ResMut<SpecializedRenderPipelines<PostProcessPipeline>>,
    post_process_pipeline: Res<PostProcessPipeline>,
    views_query: Query<(Entity, &ExtractedView)>,
) {
    for (entity, view) in &views_query {
        commands
            .entity(entity)
            .insert(Lighting2dPostProcessPipelineId(
                post_process_pipelines.specialize(
                    &pipeline_cache,
                    &post_process_pipeline,
                    Lighting2dPipelineKey { hdr: view.hdr },
                ),
            ));
    }
}

fn create_aux_texture(
    view_target: &ViewTarget,
    texture_cache: &mut TextureCache,
    render_device: &RenderDevice,
    label: &'static str,
) -> CachedTexture {
    texture_cache.get(
        render_device,
        TextureDescriptor {
            label: Some(label),
            size: view_target.main_texture().size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
    )
}

#[derive(Component)]
pub struct Lighting2dAuxiliaryTextures {
    pub sdf: CachedTexture,
    pub lighting: CachedTexture,
    pub blur_x: CachedTexture,
    pub blur_y: CachedTexture,
}

pub fn prepare_lighting_auxiliary_textures(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    view_targets: Query<(Entity, &ViewTarget)>,
) {
    for (entity, view_target) in &view_targets {
        commands.entity(entity).insert(Lighting2dAuxiliaryTextures {
            sdf: create_aux_texture(view_target, &mut texture_cache, &render_device, "sdf"),
            lighting: create_aux_texture(
                view_target,
                &mut texture_cache,
                &render_device,
                "lighting",
            ),
            blur_x: create_aux_texture(view_target, &mut texture_cache, &render_device, "blur_x"),
            blur_y: create_aux_texture(view_target, &mut texture_cache, &render_device, "blur_y"),
        });
    }
}
