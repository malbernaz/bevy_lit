use bevy::{
    color::palettes::tailwind::{GRAY_500, GRAY_700},
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

const X_EXTENT: f32 = 700.;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_scene(bsn! {
        Camera2d
        Lighting2dSettings { scale: 1.0 }
        AmbientLight2d { intensity: 0.2 }
    });

    commands.spawn_scene(bsn! {
        CursorLight
        SpotLight2d { intensity: 4.0, outer_radius: 1024.0, outer_angle: 15.0 }
        Transform {
            translation: { Vec3::new(0.0, 512.0, 0.0) },
            rotation: { Quat::from_rotation_z(-90_f32.to_radians()) },
        }
    });

    let shapes = [
        meshes.add(Circle::new(50.0)),
        meshes.add(Annulus::new(25.0, 50.0)),
        meshes.add(Capsule2d::new(25.0, 50.0)),
        meshes.add(Rhombus::new(75.0, 100.0)),
        meshes.add(Rectangle::new(50.0, 100.0)),
        meshes.add(RegularPolygon::new(50.0, 6)),
        meshes.add(Triangle2d::new(
            Vec2::Y * 50.0,
            Vec2::new(-50.0, -50.0),
            Vec2::new(50.0, -50.0),
        )),
    ];
    let color = materials.add(Color::from(GRAY_700));
    let num_shapes = shapes.len();

    let mut shape_scenes = Vec::new();
    for (i, shape) in shapes.into_iter().enumerate() {
        let material = color.clone();
        let x = -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT;
        shape_scenes.push(bsn! {
            Mesh2d(shape) MeshMaterial2d::<ColorMaterial>(material) LightOccluder2d
            Transform::from_xyz(x, 0.0, 0.0)
        });
    }
    commands.spawn_scene_list(shape_scenes);
}

fn update_cursor_light(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Lighting2dSettings>>,
    mut transform: Single<&mut Transform, With<CursorLight>>,
) {
    let (camera, camera_transform) = camera.into_inner();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate().extend(0.0))
    {
        look_at_2d(&mut transform, world_position.xy());
    }
}

pub fn look_at_2d(transform: &mut Transform, target: Vec2) {
    let delta = target - transform.translation.truncate();

    if delta.length_squared() > f32::EPSILON {
        let angle = delta.y.atan2(delta.x);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}
