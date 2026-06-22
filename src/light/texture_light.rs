use bevy::{
    asset::{embedded_asset, embedded_path, AssetPath},
    camera::visibility::{add_visibility_class, VisibilityClass},
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderType},
        sync_world::SyncToRenderWorld,
        texture::GpuImage,
    },
};

use crate::light::render::{CustomLight2dPlugin, Light2dMaterial, Light2dShaderRef, Light2dSize};

pub struct TextureLight2dPlugin;
impl Plugin for TextureLight2dPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "texture_light.wgsl");
        app.add_plugins(CustomLight2dPlugin::<TextureLight2d>::default());
    }
}

/// Represents a texture light in a 2D environment
#[derive(Component, Clone, Reflect, AsBindGroup, FromTemplate)]
#[require(SyncToRenderWorld, Transform, Visibility, VisibilityClass)]
#[component(on_add = add_visibility_class::<Self>)]
#[uniform(0, Texture2dGpuType)]
pub struct TextureLight2d {
    /// The color of the texture light
    pub color: Color,
    /// The intensity of the texture light
    pub intensity: f32,
    /// The texture of the light
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
    /// Whether the texture light should project shadows
    pub cast_shadows: bool,
}

impl Default for TextureLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
            image: Default::default(),
            cast_shadows: true,
        }
    }
}

#[derive(ShaderType)]
pub struct Texture2dGpuType {
    color: LinearRgba,
    cast_shadows: u32,
}

impl AsBindGroupShaderType<Texture2dGpuType> for TextureLight2d {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> Texture2dGpuType {
        Texture2dGpuType {
            color: self.color.to_linear() * self.intensity,
            cast_shadows: if self.cast_shadows { 1 } else { 0 },
        }
    }
}

impl Light2dMaterial for TextureLight2d {
    fn fragment_shader() -> Light2dShaderRef {
        AssetPath::from_path_buf(embedded_path!("texture_light.wgsl"))
            .with_source("embedded")
            .into()
    }

    #[inline]
    fn light_size(&self) -> Light2dSize {
        self.image.clone().into()
    }
}
