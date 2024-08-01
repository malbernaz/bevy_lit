use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    prelude::*,
    render::{
        extract_component::UniformComponentPlugin,
        gpu_component_array_buffer::GpuComponentArrayBufferPlugin,
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::SpecializedRenderPipelines,
        view::{check_visibility, prepare_view_targets, VisibilitySystems},
        Render, RenderApp, RenderSet,
    },
};

use crate::{
    extract::{
        extract_light_occluders, extract_lighting_settings, extract_point_lights,
        ExtractedLightOccluder2d, ExtractedLighting2dSettings, ExtractedPointLight2d,
    },
    pipeline::{
        Lighting2dPrepassPipelines, LightingLabel, LightingNode, PostProcessPipeline, BLUR_SHADER,
        LIGHTING_SHADER, POST_PROCESS_SHADER, SDF_SHADER, TYPES_SHADER,
        VIEW_TRANSFORMATIONS_SHADER,
    },
    prelude::{AmbientLight2d, LightOccluder2d, Lighting2dSettings, PointLight2d},
    prepare::{
        prepare_lighting_auxiliary_textures, prepare_lighting_bind_groups,
        prepare_post_process_pipelines,
    },
};

#[derive(Default)]
pub struct Lighting2dPlugin;

impl Plugin for Lighting2dPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, TYPES_SHADER, "shaders/types.wgsl", Shader::from_wgsl);
        load_internal_asset!(
            app,
            VIEW_TRANSFORMATIONS_SHADER,
            "shaders/view_transformations.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(app, SDF_SHADER, "shaders/sdf.wgsl", Shader::from_wgsl);
        load_internal_asset!(
            app,
            LIGHTING_SHADER,
            "shaders/lighting.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(app, BLUR_SHADER, "shaders/blur.wgsl", Shader::from_wgsl);
        load_internal_asset!(
            app,
            POST_PROCESS_SHADER,
            "shaders/post_process.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins((
            UniformComponentPlugin::<ExtractedLighting2dSettings>::default(),
            GpuComponentArrayBufferPlugin::<ExtractedPointLight2d>::default(),
            GpuComponentArrayBufferPlugin::<ExtractedLightOccluder2d>::default(),
        ))
        .register_type::<AmbientLight2d>()
        .register_type::<PointLight2d>()
        .register_type::<LightOccluder2d>()
        .register_type::<Lighting2dSettings>()
        .add_systems(
            PostUpdate,
            check_visibility::<Or<(With<PointLight2d>, With<LightOccluder2d>)>>
                .in_set(VisibilitySystems::CheckVisibility),
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<PostProcessPipeline>>()
            .add_systems(
                ExtractSchedule,
                (
                    extract_lighting_settings,
                    extract_light_occluders,
                    extract_point_lights,
                ),
            )
            .add_systems(
                Render,
                (
                    prepare_lighting_auxiliary_textures
                        .after(prepare_view_targets)
                        .in_set(RenderSet::ManageViews),
                    prepare_post_process_pipelines.in_set(RenderSet::Prepare),
                    prepare_lighting_bind_groups.in_set(RenderSet::PrepareBindGroups),
                ),
            )
            .add_render_graph_node::<ViewNodeRunner<LightingNode>>(Core2d, LightingLabel)
            .add_render_graph_edges(Core2d, (Node2d::EndMainPass, LightingLabel));
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<Lighting2dPrepassPipelines>()
            .init_resource::<PostProcessPipeline>();
    }
}
