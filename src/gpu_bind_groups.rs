use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            BindGroup, BindGroupEntries, CachedRenderPipelineId, PipelineCache,
            SpecializedRenderPipelines,
        },
        renderer::RenderDevice,
        texture::GpuImage,
        view::{ExtractedView, ViewUniforms},
    },
};

use crate::{
    extract::{
        ExtractedAmbientLight2d, ExtractedLightOccluder2d, ExtractedLighting2dSettings,
        ExtractedPointLight2d,
    },
    gpu_resources::{LightingArrayBuffer, LightingUniform},
    pipeline::{LightingPipelines, PostProcessKey, PostProcessPipeline},
    surfaces::{BLUR_SURFACE, SDF_SURFACE, SURFACE},
};

#[derive(Resource, Debug)]
pub struct Lighting2dSurfaceBindGroups {
    pub sdf: BindGroup,
    pub lighting: BindGroup,
    pub blur: BindGroup,
}

pub fn prepare_lighting_assets(
    mut commands: Commands,
    pipeline: Res<LightingPipelines>,
    render_device: Res<RenderDevice>,
    images: ResMut<RenderAssets<GpuImage>>,
    view_uniforms: Res<ViewUniforms>,
    ambient_light: Res<LightingUniform<ExtractedAmbientLight2d>>,
    lighting_settings: Res<LightingUniform<ExtractedLighting2dSettings>>,
    point_lights: Res<LightingArrayBuffer<ExtractedPointLight2d>>,
    light_occluders: Res<LightingArrayBuffer<ExtractedLightOccluder2d>>,
) {
    let (
        Some(view_uniform),
        Some(lighting_settings),
        Some(ambient_light),
        Some(light_occluders),
        Some(point_lights),
        Some(sdf_surface),
        Some(surface),
        Some(blur_surface),
    ) = (
        view_uniforms.uniforms.binding(),
        lighting_settings.binding(),
        ambient_light.binding(),
        light_occluders.binding(),
        point_lights.binding(),
        images.get(&SDF_SURFACE),
        images.get(&SURFACE),
        images.get(&BLUR_SURFACE),
    )
    else {
        return;
    };

    commands.insert_resource(Lighting2dSurfaceBindGroups {
        sdf: render_device.create_bind_group(
            "sdf_bind_group",
            &pipeline.sdf_layout,
            &BindGroupEntries::sequential((
                view_uniform.clone(),
                lighting_settings.clone(),
                light_occluders,
                &sdf_surface.texture_view,
            )),
        ),
        lighting: render_device.create_bind_group(
            "lighting2d_bind_group",
            &pipeline.lighting_layout,
            &BindGroupEntries::sequential((
                view_uniform,
                lighting_settings.clone(),
                ambient_light,
                point_lights,
                &surface.texture_view,
                &sdf_surface.texture_view,
                &sdf_surface.sampler,
            )),
        ),
        blur: render_device.create_bind_group(
            "blur_bind_group",
            &pipeline.blur_layout,
            &BindGroupEntries::sequential((
                lighting_settings,
                &blur_surface.texture_view,
                &surface.texture_view,
                &surface.sampler,
            )),
        ),
    });
}

#[derive(Component)]
pub struct PostProcessPipelineId {
    pub id: CachedRenderPipelineId,
}

pub fn prepare_post_process_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<PostProcessPipeline>>,
    pipeline: Res<PostProcessPipeline>,
    view_targets: Query<(Entity, &ExtractedView)>,
) {
    for (entity, view) in view_targets.iter() {
        commands.entity(entity).insert(PostProcessPipelineId {
            id: pipelines.specialize(&pipeline_cache, &pipeline, PostProcessKey { hdr: view.hdr }),
        });
    }
}
