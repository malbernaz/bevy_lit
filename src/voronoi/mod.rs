use bevy::{
    asset::{embedded_asset, load_embedded_asset, AssetEventSystems},
    core_pipeline::FullscreenShader,
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    math::FloatOrd,
    mesh::Mesh2d,
    mesh::MeshVertexBufferLayoutRef,
    platform::collections::HashSet,
    prelude::*,
    render::{
        batching::no_gpu_preprocessing::batch_and_prepare_sorted_render_phase,
        camera::{DirtySpecializationSystems, DirtySpecializations, PendingQueues},
        extract_component::ExtractComponentPlugin,
        mesh::RenderMesh,
        render_asset::{prepare_assets, RenderAssets},
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, PipelineCache,
            RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor, ShaderStages,
            SpecializedMeshPipeline, SpecializedMeshPipelineError, SpecializedMeshPipelines,
            TextureFormat, TextureSampleType,
        },
        renderer::RenderDevice,
        sync_world::{MainEntity, MainEntityHashMap},
        texture::{FallbackImage, GpuImage},
        view::{ExtractedView, RenderVisibleEntities, RetainedViewEntity},
        Extract, Render, RenderApp, RenderStartup, RenderSystems,
    },
    sprite_render::{
        init_mesh_2d_pipeline, DrawMesh2d, EntitiesNeedingSpecialization, Mesh2dPipeline,
        Mesh2dPipelineKey, RenderMesh2dInstances, SetMesh2dBindGroup, SetMesh2dViewBindGroup,
        SpecializedMaterial2dPipelineCache, ViewKeyCache,
    },
    utils::Parallel,
};

use crate::{
    occlusion::LightOccluder2d,
    render::{extract_light2d_phases, VoronoiPhase},
    wrappers::UVec2Uniform,
};

pub struct Voronoi2dPlugin;
impl Plugin for Voronoi2dPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "mask.wgsl");
        embedded_asset!(app, "flood_seed.wgsl");
        embedded_asset!(app, "flood.wgsl");

        app.add_plugins(ExtractComponentPlugin::<LightOccluder2d>::default())
            .init_resource::<EntitiesNeedingSpecialization<LightOccluder2d>>()
            .add_systems(
                PostUpdate,
                check_occluders_needing_specialization.after(AssetEventSystems),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedMeshPipelines<MaskPipeline>>()
            .init_resource::<RenderVoronoiMaterials>()
            .init_resource::<MaskMaterialBindGroups>()
            .init_resource::<DrawFunctions<VoronoiPhase>>()
            .init_resource::<SpecializedMaterial2dPipelineCache<LightOccluder2d>>()
            .init_resource::<PendingMaskQueues>()
            .add_render_command::<VoronoiPhase, DrawMaskMesh>()
            .add_systems(
                ExtractSchedule,
                (
                    extract_occluder_specialization
                        .in_set(DirtySpecializationSystems::CheckForChanges),
                    extract_occluder_specializations_removed
                        .in_set(DirtySpecializationSystems::CheckForRemovals),
                    extract_voronoi_materials,
                )
                    .after(extract_light2d_phases),
            )
            .add_systems(
                RenderStartup,
                (
                    init_mask_pipeline.after(init_mesh_2d_pipeline),
                    init_flood_pipeline,
                ),
            )
            .add_systems(
                Render,
                (
                    prepare_pending_mask_queues.in_set(RenderSystems::Specialize),
                    specialize_mask_meshes
                        .in_set(RenderSystems::Specialize)
                        .after(prepare_assets::<RenderMesh>)
                        .after(prepare_pending_mask_queues),
                    queue_mask_meshes
                        .in_set(RenderSystems::QueueMeshes)
                        .after(prepare_assets::<RenderMesh>),
                    batch_and_prepare_sorted_render_phase::<VoronoiPhase, Mesh2dPipeline>
                        .in_set(RenderSystems::PrepareResources),
                    prepare_mask_material_bind_groups.in_set(RenderSystems::PrepareBindGroups),
                ),
            );
    }
}

pub fn check_occluders_needing_specialization(
    needs_specialization: Query<
        Entity,
        (
            Or<(
                Changed<Mesh2d>,
                AssetChanged<Mesh2d>,
                Changed<LightOccluder2d>,
                AssetChanged<LightOccluder2d>,
                Changed<GlobalTransform>,
            )>,
            With<LightOccluder2d>,
        ),
    >,
    mut par_local: Local<Parallel<Vec<Entity>>>,
    mut entities_needing_specialization: ResMut<EntitiesNeedingSpecialization<LightOccluder2d>>,
    mut removed_mesh_2d_components: RemovedComponents<Mesh2d>,
    mut removed_mask_components: RemovedComponents<LightOccluder2d>,
) {
    entities_needing_specialization.changed.clear();
    entities_needing_specialization.removed.clear();

    needs_specialization
        .par_iter()
        .for_each(|entity| par_local.borrow_local_mut().push(entity));

    par_local.drain_into(&mut entities_needing_specialization.changed);

    for entity in removed_mesh_2d_components
        .read()
        .chain(removed_mask_components.read())
    {
        entities_needing_specialization.removed.push(entity);
    }
}

/// Temporarily stores voronoi meshes that couldn't be specialized yet because
/// their mesh/material hadn't loaded.
///
/// See the documentation for [`PendingQueues`] for more information.
#[derive(Default, Deref, DerefMut, Resource)]
pub struct PendingMaskQueues(pub PendingQueues);

pub fn prepare_pending_mask_queues(
    mut pending_mask_queues: ResMut<PendingMaskQueues>,
    views: Query<&ExtractedView>,
) {
    let mut all_views: HashSet<RetainedViewEntity> = HashSet::default();
    for view in &views {
        all_views.insert(view.retained_view_entity);
        pending_mask_queues.prepare_for_new_frame(view.retained_view_entity);
    }
    pending_mask_queues.expire_stale_views(&all_views);
}

/// Drains entities that need their specializations updated from the main-world
/// [`EntitiesNeedingSpecialization`] resource into the render-world
/// [`DirtySpecializations`] table.
pub fn extract_occluder_specialization(
    entities_needing_specialization: Extract<Res<EntitiesNeedingSpecialization<LightOccluder2d>>>,
    mut dirty_specializations: ResMut<DirtySpecializations>,
) {
    for entity in entities_needing_specialization.changed.iter() {
        dirty_specializations
            .changed_renderables
            .insert(MainEntity::from(*entity));
    }
}

/// Drains entities that need their specializations removed into the
/// [`DirtySpecializations`] table.
pub fn extract_occluder_specializations_removed(
    entities_needing_specialization: Extract<Res<EntitiesNeedingSpecialization<LightOccluder2d>>>,
    mut dirty_specializations: ResMut<DirtySpecializations>,
) {
    for entity in entities_needing_specialization.removed.iter() {
        dirty_specializations
            .removed_renderables
            .insert(MainEntity::from(*entity));
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct RenderVoronoiMaterials(MainEntityHashMap<AssetId<Image>>);

fn extract_voronoi_materials(
    mut render_voronoi_instances: ResMut<RenderVoronoiMaterials>,
    query: Extract<Query<(Entity, &ViewVisibility, &LightOccluder2d), With<Mesh2d>>>,
) {
    render_voronoi_instances.clear();

    for (entity, view_visibility, material) in &query {
        if view_visibility.get() {
            render_voronoi_instances.insert(entity.into(), material.into());
        }
    }
}

#[derive(Resource)]
pub struct MaskPipeline {
    pub mesh_pipeline: Mesh2dPipeline,
    pub material_layout: BindGroupLayoutDescriptor,
    pub shader: Handle<Shader>,
}

impl SpecializedMeshPipeline for MaskPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let descriptor = self.mesh_pipeline.specialize(key, &layout)?;

        let mut mesh_layout = descriptor.layout.clone();
        mesh_layout.push(self.material_layout.clone());

        Ok(RenderPipelineDescriptor {
            label: Some("mask_pipeline".into()),
            layout: mesh_layout,
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            depth_stencil: None,
            multisample: Default::default(),
            ..descriptor
        })
    }
}

pub fn init_mask_pipeline(
    mut commands: Commands,
    mesh_2d_pipeline: Res<Mesh2dPipeline>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(MaskPipeline {
        mesh_pipeline: mesh_2d_pipeline.clone(),
        shader: load_embedded_asset!(asset_server.as_ref(), "mask.wgsl"),
        material_layout: BindGroupLayoutDescriptor::new(
            "mask_material_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        ),
    });
}

pub fn specialize_mask_meshes(
    render_meshes: Res<RenderAssets<RenderMesh>>,
    pipeline_cache: Res<PipelineCache>,
    mut render_mesh_instances: ResMut<RenderMesh2dInstances>,
    mut mask_pipelines: ResMut<SpecializedMeshPipelines<MaskPipeline>>,
    mask_pipeline: Res<MaskPipeline>,
    view_key_cache: Res<ViewKeyCache>,
    views: Query<(&MainEntity, &ExtractedView, &RenderVisibleEntities)>,
    render_material_instances: Res<RenderVoronoiMaterials>,
    dirty_specializations: Res<DirtySpecializations>,
    mut pending_mask_queues: ResMut<PendingMaskQueues>,
    mut specialized_material_pipeline_cache: ResMut<
        SpecializedMaterial2dPipelineCache<LightOccluder2d>,
    >,
) {
    if render_material_instances.is_empty() {
        return;
    }

    for (view_entity, view, visible_entities) in &views {
        let Some(views_key) = view_key_cache.get(view_entity) else {
            continue;
        };
        let view_key = *views_key;

        let view_specialized_material_pipeline_cache = specialized_material_pipeline_cache
            .entry(*view_entity)
            .or_default();

        let Some(visible_entities) = visible_entities.get::<Mesh2d>() else {
            continue;
        };

        // Remove cached pipeline IDs corresponding to entities that either
        // have been removed or need to be re-specialized.
        if dirty_specializations.must_wipe_specializations_for_view(view.retained_view_entity) {
            view_specialized_material_pipeline_cache.clear();
        } else {
            for &renderable_entity in dirty_specializations.iter_to_despecialize() {
                view_specialized_material_pipeline_cache.remove(&renderable_entity);
            }
        }

        let Some(view_pending_mask_queues) =
            pending_mask_queues.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        // Now process all occluder meshes that need to be re-specialized.
        for (render_entity, visible_entity) in dirty_specializations.iter_to_specialize(
            view.retained_view_entity,
            visible_entities,
            &view_pending_mask_queues.prev_frame,
        ) {
            if view_specialized_material_pipeline_cache.contains_key(visible_entity) {
                continue;
            }

            if !render_material_instances.contains_key(visible_entity) {
                // Entity doesn't have a light occluder. Skip it.
                continue;
            }

            let Some(mesh_instance) = render_mesh_instances.get_mut(visible_entity) else {
                continue;
            };
            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                // We couldn't fetch the mesh, probably because it hasn't been
                // loaded yet. Add the entity to the list of pending mask meshes
                // and bail.
                view_pending_mask_queues
                    .current_frame
                    .insert((*render_entity, *visible_entity));
                continue;
            };

            let mesh_key = view_key
                | Mesh2dPipelineKey::from_primitive_topology_and_strip_index(
                    mesh.primitive_topology(),
                    mesh.index_format(),
                );

            let pipeline_id =
                mask_pipelines.specialize(&pipeline_cache, &mask_pipeline, mesh_key, &mesh.layout);
            let pipeline_id = match pipeline_id {
                Ok(id) => id,
                Err(err) => {
                    error!("{}", err);
                    continue;
                }
            };

            view_specialized_material_pipeline_cache.insert(*visible_entity, pipeline_id);
        }
    }
}

pub fn queue_mask_meshes(
    mask_draw_functions: Res<DrawFunctions<VoronoiPhase>>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMesh2dInstances>,
    mut mask_render_phase: ResMut<ViewSortedRenderPhases<VoronoiPhase>>,
    views: Query<(&MainEntity, &ExtractedView, &RenderVisibleEntities)>,
    render_material_instances: Res<RenderVoronoiMaterials>,
    dirty_specializations: Res<DirtySpecializations>,
    mut pending_mask_queues: ResMut<PendingMaskQueues>,
    specialized_material_pipeline_cache: ResMut<
        SpecializedMaterial2dPipelineCache<LightOccluder2d>,
    >,
) {
    if render_material_instances.is_empty() {
        return;
    }

    for (view_entity, view, visible_entities) in &views {
        let Some(view_specialized_material_pipeline_cache) =
            specialized_material_pipeline_cache.get(view_entity)
        else {
            continue;
        };

        let Some(mask_phase) = mask_render_phase.get_mut(&view.retained_view_entity) else {
            continue;
        };

        let Some(visible_entities) = visible_entities.get::<Mesh2d>() else {
            continue;
        };

        let view_pending_mask_queues = pending_mask_queues
            .get_mut(&view.retained_view_entity)
            .expect(
            "View pending mask queues should have been created in `prepare_pending_mask_queues`",
        );

        let draw_mask_mesh = mask_draw_functions.read().id::<DrawMaskMesh>();

        // Remove entities that became invisible or fully lost their occluder from
        // the render phase. Entities that are also in `changed_renderables` are
        // switching and will be handled by the inline dequeue in the queue loop
        // below.
        for main_entity in visible_entities
            .removed_entities
            .iter()
            .map(|(_, main_entity)| main_entity)
            .chain(
                dirty_specializations
                    .removed_renderables
                    .iter()
                    .filter(|e| !dirty_specializations.changed_renderables.contains(*e)),
            )
        {
            mask_phase.remove(Entity::PLACEHOLDER, *main_entity);
        }

        // Now iterate over all newly-visible entities and those that need
        // specialization.
        for (render_entity, visible_entity) in dirty_specializations.iter_to_queue(
            view.retained_view_entity,
            visible_entities,
            &view_pending_mask_queues.prev_frame,
        ) {
            let Some(pipeline_id) = view_specialized_material_pipeline_cache
                .get(visible_entity)
                .copied()
            else {
                continue;
            };

            if !render_material_instances.contains_key(visible_entity) {
                continue;
            }

            let Some(mesh_instance) = render_mesh_instances.get(visible_entity) else {
                continue;
            };
            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                // We couldn't fetch the mesh, probably because it hasn't been
                // loaded yet. Add the entity to the list of pending mask meshes
                // and bail.
                view_pending_mask_queues
                    .current_frame
                    .insert((*render_entity, *visible_entity));
                continue;
            };

            // Remove old phase item before re-adding. This handles key changes
            // and is safe even if the entity wasn't previously queued.
            mask_phase.remove(Entity::PLACEHOLDER, *visible_entity);

            // Occluders persist between frames; the change-list `remove` above and
            // the per-removed-entity dequeue handle additions/removals.
            //
            // Like Bevy's `Transparent2d`, we use `Entity::PLACEHOLDER` as the
            // render entity so that `DirtySpecializations`-driven dequeues (which
            // only track main-world entities) match the stored item key.
            mask_phase.add_retained(VoronoiPhase {
                sort_key: FloatOrd(mesh_instance.transforms.world_from_local.translation.z),
                pipeline: pipeline_id,
                draw_function: draw_mask_mesh,
                entity: (Entity::PLACEHOLDER, *visible_entity),
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: mesh.indexed(),
            });
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct MaskMaterialBindGroups(MainEntityHashMap<BindGroup>);

pub fn prepare_mask_material_bind_groups(
    render_device: Res<RenderDevice>,
    pipeline: Res<MaskPipeline>,
    images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    voronoi_materials: Res<RenderVoronoiMaterials>,
    pipeline_cache: Res<PipelineCache>,
    mut bind_groups: ResMut<MaskMaterialBindGroups>,
) {
    // Only update bind groups for entities that have changed or are new
    bind_groups.retain(|entity, _| voronoi_materials.contains_key(entity));

    for (entity, alpha_mask) in voronoi_materials.iter() {
        let alpha_mask_image = if let Some(image) = images.get(*alpha_mask) {
            image
        } else {
            &fallback_image.d2
        };
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let bind_group = render_device.create_bind_group(
            "mask_material_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline.material_layout),
            &BindGroupEntries::sequential((&alpha_mask_image.texture_view, &sampler)),
        );
        bind_groups.insert(*entity, bind_group);
    }
}

pub type DrawMaskMesh = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    SetMaskMaterialBindGroup<2>,
    DrawMesh2d,
);

pub struct SetMaskMaterialBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetMaskMaterialBindGroup<I> {
    type Param = SRes<MaskMaterialBindGroups>;
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let bind_groups = bind_groups.into_inner();
        let Some(bind_group) = bind_groups.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &bind_group, &[]);
        RenderCommandResult::Success
    }
}

#[derive(Resource)]
pub struct FloodPipeline {
    pub seed_layout: BindGroupLayoutDescriptor,
    pub seed_pipeline: CachedRenderPipelineId,
    pub layout: BindGroupLayoutDescriptor,
    pub pipeline: CachedRenderPipelineId,
}

pub fn init_flood_pipeline(
    mut commands: Commands,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {
    let seed_layout = BindGroupLayoutDescriptor::new(
        "flood_seed_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    );

    let fullscreen_vertex_state = fullscreen_shader.to_vertex_state();

    let seed_pipeline = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("flood_seed_pipeline".into()),
        layout: vec![seed_layout.clone()],
        vertex: fullscreen_vertex_state.clone(),
        fragment: Some(FragmentState {
            shader: load_embedded_asset!(asset_server.as_ref(), "flood_seed.wgsl"),
            shader_defs: vec![],
            entry_point: Some("fragment".into()),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba16Float,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        }),
        ..default()
    });

    let layout = BindGroupLayoutDescriptor::new(
        "flood_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<UVec2Uniform>(false),
            ),
        ),
    );

    let pipeline = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("flood_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: fullscreen_vertex_state,
        fragment: Some(FragmentState {
            shader: load_embedded_asset!(asset_server.as_ref(), "flood.wgsl"),
            shader_defs: vec![],
            entry_point: Some("fragment".into()),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba16Float,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        }),
        ..default()
    });

    commands.insert_resource(FloodPipeline {
        seed_pipeline,
        seed_layout,
        layout,
        pipeline,
    });
}
