use bevy::{camera::ScalingMode, color::palettes::tailwind::*, prelude::*, window::PrimaryWindow};
use bevy_lit::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            Lighting2dPlugin,
        ))
        .insert_resource(ClearColor(Color::from(GRAY_600)))
        .add_systems(Startup, setup)
        .add_systems(Update, update_cursor_light)
        .run();
}

#[derive(Component, Default, Clone)]
struct CursorLight;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut camera = commands.spawn_scene(bsn! {
        Camera2d
        Lighting2dSettings {
            raymarch: RaymarchSettings { max_steps: 32, jitter_contrib: 0.0, sharpness: 64. },
            scale: 0.125,
        }
        AmbientLight2d { intensity: 0.2, color: { Color::from(BLUE_300) } }
    });
    camera.insert(Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedHorizontal {
            viewport_width: 320.0,
        },
        ..OrthographicProjection::default_2d()
    }));

    let light_mask = asset_server.load("light_mask.png");
    commands.spawn_scene(bsn! {
        CursorLight
        TextureLight2d {
            image: light_mask,
            color: { Color::from(YELLOW_400) },
            intensity: 1.0,
            cast_shadows: true,
        }
        Sprite { custom_size: { Some(Vec2::splat(8.0)) } }
    });

    let tile = meshes.add(Rectangle::from_length(16.));
    let material = materials.add(Color::from(GRAY_800));

    let tile_a = tile.clone();
    let material_a = material.clone();
    commands.spawn_scene(bsn! {
        Mesh2d(tile_a) MeshMaterial2d::<ColorMaterial>(material_a) LightOccluder2d
        Transform::from_xyz(-16.0, 0.0, 0.0)
    });
    commands.spawn_scene(bsn! {
        Mesh2d(tile) MeshMaterial2d::<ColorMaterial>(material) LightOccluder2d
        Transform::from_xyz(16.0, 0.0, 0.0)
    });
}

fn update_cursor_light(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Lighting2dSettings>>,
    mut point_light_transform: Single<&mut Transform, With<CursorLight>>,
) {
    let (camera, camera_transform) = camera.into_inner();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate().extend(0.0))
    {
        point_light_transform.translation = world_position;
    }
}
