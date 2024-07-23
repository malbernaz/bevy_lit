use bevy::{
    prelude::*,
    render::{
        render_resource::{
            binding_types::{storage_buffer_read_only, uniform_buffer},
            encase::internal::WriteInto,
            BindGroupLayoutEntryBuilder, BindingResource, GpuArrayBufferable, ShaderType,
            StorageBuffer, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::extract::{
    ExtractedAmbientLight2d, ExtractedLightOccluder2d, ExtractedLighting2dSettings,
    ExtractedPointLight2d,
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
