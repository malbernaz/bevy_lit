use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            BindGroupEntries, CachedRenderPipelineId, GpuArrayBuffer, PipelineCache,
            SpecializedRenderPipelines,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        view::{ExtractedView, ViewUniforms},
    },
};

use super::{
    extract::{
        AmbientLight2dUniform, ExtractedLightOccluder2d, ExtractedPointLight2d,
        Lighting2dSettingsUniform,
    },
    pipeline::{LightingPipelines, PostProcessKey, PostProcessPipeline},
    resources::Lighting2dSurfaceBindGroups,
    surfaces::{BLUR_SURFACE, SDF_SURFACE, SURFACE},
};

#[allow(clippy::too_many_arguments)]
pub fn prepare_lighting_assets(
    mut commands: Commands,
    pipeline: Res<LightingPipelines>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    images: ResMut<RenderAssets<GpuImage>>,
    view_uniforms: Res<ViewUniforms>,
    mut ambient_light: ResMut<AmbientLight2dUniform>,
    mut lighting_settings: ResMut<Lighting2dSettingsUniform>,
    point_lights: Res<GpuArrayBuffer<ExtractedPointLight2d>>,
    light_occluders: Res<GpuArrayBuffer<ExtractedLightOccluder2d>>,
) {
    ambient_light.write_buffer(&render_device, &render_queue);
    lighting_settings.write_buffer(&render_device, &render_queue);

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
