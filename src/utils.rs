use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::{ImageSampler, ImageSamplerDescriptor},
    },
};

pub fn create_lighting_surface(size: UVec2) -> Image {
    let size = Extent3d {
        width: size.x,
        height: size.y,
        ..default()
    };
    let mut surface = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        sampler: ImageSampler::Descriptor(ImageSamplerDescriptor::linear()),
        ..default()
    };
    surface.resize(size);
    surface
}

pub fn workgroup_grid_from_image_size(image_size: UVec2, workgroup_size: u32) -> UVec2 {
    let x_pad = image_size.x % workgroup_size;
    let y_pad = image_size.y % workgroup_size;

    UVec2::new(
        if x_pad == 0 {
            image_size.x
        } else {
            image_size.x + workgroup_size - x_pad
        },
        if y_pad == 0 {
            image_size.y
        } else {
            image_size.y + workgroup_size - y_pad
        },
    ) / workgroup_size
}
