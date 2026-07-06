use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::{add_visibility_class, VisibilityClass},
    color::palettes::tailwind::*,
    prelude::*,
    render::{
        render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat},
        sync_world::SyncToRenderWorld,
    },
    window::PrimaryWindow,
};
use bevy_lit::prelude::*;

#[derive(Component, Clone, Reflect, AsBindGroup, FromTemplate)]
#[require(SyncToRenderWorld, Transform, Visibility, VisibilityClass)]
#[component(on_add = add_visibility_class::<Self>)]
pub struct ToonLight2d {
    #[texture(0, dimension = "1d")]
    pub gradient_map: Handle<Image>,
    // We need 16-byte alignment for all fields of the `AsBindGroup` struct if we want to support WebGL2.
    // This is why we are using `Vec4` here instead of `f32`.
    #[uniform(1)]
    pub radius: Vec4,
    #[uniform(2)]
    pub color: LinearRgba,
}

impl Default for ToonLight2d {
    fn default() -> Self {
        Self {
            gradient_map: Default::default(),
            radius: Vec4::splat(200.0),
            color: LinearRgba::WHITE,
        }
    }
}

impl Light2dMaterial for ToonLight2d {
    fn fragment_shader() -> Light2dShaderRef {
        "toon_light.wgsl".into()
    }

    fn light_size(&self) -> Light2dSize {
        (self.radius.x * 2.0).into()
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Lighting2dPlugin,
            CustomLight2dPlugin::<ToonLight2d>::default(),
        ))
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, setup)
        .add_systems(Update, update_cursor_position)
        .run();
}

#[derive(Component, Default, Clone)]
struct OccluderCursor;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn_scene(bsn! {
        Camera2d
        Lighting2dSettings {
            raymarch: RaymarchSettings { jitter_contrib: 0.0, sharpness: 1000.0 }
        }
        AmbientLight2d { intensity: 0.02 }
    });

    let gradient_map = images.add(generate_toon_gradient(5));

    commands.spawn_scene(bsn! {
        ToonLight2d {
            gradient_map: gradient_map,
            radius: Vec4::splat(300.0),
            color: { Color::from(YELLOW_100).to_linear() },
        }
    });

    let occluder_mesh = meshes.add(Circle::new(25.0));

    commands.spawn_scene(bsn! {
        OccluderCursor
        Mesh2d(occluder_mesh)
        LightOccluder2d
    });
}

fn update_cursor_position(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Lighting2dSettings>>,
    mut cursor_transform: Single<&mut Transform, With<OccluderCursor>>,
) {
    let (camera, camera_transform) = camera.into_inner();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate().extend(0.0))
    {
        cursor_transform.translation = world_position;
    }
}

fn generate_toon_gradient(levels: usize) -> Image {
    let mut data = Vec::with_capacity(levels);

    for i in 0..levels {
        let value = i as f32 / (levels - 1) as f32;
        data.push((value * 255.0) as u8);
    }

    Image::new_fill(
        Extent3d {
            width: levels as u32,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D1,
        &data,
        TextureFormat::R8Unorm,
        RenderAssetUsages::default(),
    )
}
