use bevy::{
    prelude::*,
    render::{
        render_resource::{
            BindingResource, GpuArrayBufferable, ShaderType, StorageBuffer, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::extract::{ExtractedLightOccluder2d, ExtractedPointLight2d};

#[derive(Clone, ShaderType)]
pub struct GpuAmbientLight2d {
    pub color: Vec4,
}

#[derive(Resource)]
pub struct AmbientLight2dUniform {
    pub uniform: UniformBuffer<GpuAmbientLight2d>,
}

impl AmbientLight2dUniform {
    pub fn new(ambient_light: GpuAmbientLight2d) -> Self {
        Self {
            uniform: UniformBuffer::from(ambient_light.clone()),
        }
    }

    pub fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.uniform.write_buffer(render_device, render_queue);
    }

    pub fn binding(&self) -> Option<BindingResource> {
        self.uniform.binding()
    }
}

#[derive(Clone, ShaderType)]
pub struct GpuLighting2dGpuSettings {
    pub blur_coc: f32,
    pub viewport: UVec2,
}

#[derive(Resource)]
pub struct Lighting2dSettingsUniform {
    pub uniform: UniformBuffer<GpuLighting2dGpuSettings>,
}

impl Lighting2dSettingsUniform {
    pub fn new(settings: GpuLighting2dGpuSettings) -> Self {
        Self {
            uniform: UniformBuffer::from(settings.clone()),
        }
    }

    pub fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.uniform.write_buffer(render_device, render_queue);
    }

    pub fn binding(&self) -> Option<BindingResource> {
        self.uniform.binding()
    }
}

#[derive(Resource)]
pub enum LightingArrayBuffer<T: GpuArrayBufferable> {
    Uniform(UniformBuffer<Vec<T>>),
    Storage(StorageBuffer<Vec<T>>),
}

impl<T: GpuArrayBufferable> LightingArrayBuffer<T> {
    pub fn new(value: Vec<T>, device: &RenderDevice) -> Self {
        let limits = device.limits();
        if limits.max_storage_buffers_per_shader_stage == 0 {
            LightingArrayBuffer::Uniform(UniformBuffer::from(value))
        } else {
            LightingArrayBuffer::Storage(StorageBuffer::from(value))
        }
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        match self {
            LightingArrayBuffer::Uniform(buffer) => buffer.write_buffer(device, queue),
            LightingArrayBuffer::Storage(buffer) => buffer.write_buffer(device, queue),
        }
    }

    pub fn binding(&self) -> Option<BindingResource> {
        match self {
            LightingArrayBuffer::Uniform(buffer) => buffer.binding(),
            LightingArrayBuffer::Storage(buffer) => buffer.binding(),
        }
    }
}

pub fn prepare_gpu_resources(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut ambient_light: ResMut<AmbientLight2dUniform>,
    mut lighting_settings: ResMut<Lighting2dSettingsUniform>,
    point_lights_query: Query<&ExtractedPointLight2d>,
    light_occluders_query: Query<&ExtractedLightOccluder2d>,
) {
    ambient_light.write_buffer(&render_device, &render_queue);
    lighting_settings.write_buffer(&render_device, &render_queue);

    let mut gpu_point_lights = LightingArrayBuffer::<ExtractedPointLight2d>::new(
        point_lights_query.iter().cloned().collect::<Vec<_>>(),
        &render_device,
    );
    gpu_point_lights.write_buffer(&render_device, &render_queue);
    commands.insert_resource(gpu_point_lights);

    let mut gpu_light_occluders = LightingArrayBuffer::<ExtractedLightOccluder2d>::new(
        light_occluders_query.iter().cloned().collect::<Vec<_>>(),
        &render_device,
    );
    gpu_light_occluders.write_buffer(&render_device, &render_queue);
    commands.insert_resource(gpu_light_occluders);
}
