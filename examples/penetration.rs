use bevy::{
    color::palettes::tailwind::{GRAY_200, GRAY_500},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_lit::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Lighting2dPlugin))
        .insert_resource(ClearColor(Color::from(GRAY_500)))
        .add_systems(Startup, setup)
        .add_systems(Update, update_cursor_light)
        .run();
}

#[derive(Component, Default, Clone)]
struct CursorLight;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_scene(bsn! {
        Camera2d
        Lighting2dSettings {
            penetration: PenetrationSettings {
                max: 20.0,
                intensity: 1.0,
                falloff: 1.0,
                sample_directions: 16,
                sample_steps: 8,
            },
        }
        AmbientLight2d { intensity: 0.2 }
    });

    commands.spawn_scene(bsn! {
        PointLight2d { intensity: 4.0, outer_radius: 512.0 }
        CursorLight
        Transform { rotation: { Quat::from_rotation_z(-90_f32.to_radians()) } }
    });

    let rect = meshes.add(Rectangle::from_length(100.));
    let material = materials.add(Color::from(GRAY_200));

    let rect_a = rect.clone();
    let material_a = material.clone();
    commands.spawn_scene(bsn! {
        Mesh2d(rect_a) MeshMaterial2d::<ColorMaterial>(material_a) LightOccluder2d
        Transform::from_xyz(-100., 0., 0.)
    });

    commands.spawn_scene(bsn! {
        Mesh2d(rect) MeshMaterial2d::<ColorMaterial>(material) LightOccluder2d
        Transform::from_xyz(100., 0., 0.)
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
