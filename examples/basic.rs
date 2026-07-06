use bevy::{
    color::palettes::tailwind::{BLUE_300, BLUE_600, GRAY_200, GRAY_700, YELLOW_600},
    input::mouse::MouseButtonInput,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_lit::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Lighting2dPlugin))
        .insert_resource(ClearColor(Color::from(GRAY_200)))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_cursor_light, despawn_shapes))
        .add_systems(FixedUpdate, update_moving_lights)
        .run();
}

#[derive(Component, Default, Clone)]
struct CursorLight;

#[derive(Component, Default, Clone)]
struct MovingLights;

const X_EXTENT: f32 = 700.;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_scene(bsn! {
        Camera2d
        Lighting2dSettings {
            blur: 4,
            edge_intensity: 8.0,
            raymarch: RaymarchSettings { max_steps: 32, jitter_contrib: 0.5, sharpness: 10. },
        }
        AmbientLight2d { intensity: 0.1, color: { Color::from(BLUE_300) } }
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
            Mesh2d(shape)
            MeshMaterial2d::<ColorMaterial>(material)
            LightOccluder2d
            Transform::from_xyz(x, 0.0, 0.0)
        });
    }
    commands.spawn_scene_list(shape_scenes);

    commands.spawn_scene(bsn! {
        MovingLights
        Transform
        Visibility
        Children [
            PointLight2d {
                color: { Color::from(BLUE_600) },
                intensity: 2.0,
                outer_radius: 1100.0,
                falloff: 3.0,
            } Transform::from_xyz(-X_EXTENT + 50. / 2., 0.0, 0.0),
            PointLight2d {
                color: { Color::from(BLUE_600) },
                intensity: 2.0,
                outer_radius: 1100.0,
                falloff: 3.0,
            } Transform::from_xyz(X_EXTENT + 50. / 2., 0.0, 0.0),
        ]
    });

    commands.spawn_scene(bsn! {
        CursorLight
        PointLight2d {
            color: { Color::from(YELLOW_600) },
            intensity: 2.0,
            outer_radius: 400.0,
            falloff: 10.0,
        }
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

fn despawn_shapes(
    mut commands: Commands,
    lights: Query<Entity, With<PointLight2d>>,
    mut mouse_button_events: MessageReader<MouseButtonInput>,
) {
    // Check if any mouse button was pressed
    for event in mouse_button_events.read() {
        if event.state.is_pressed() {
            // Despawn one shape per click
            if let Some(entity) = lights.iter().next() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn update_moving_lights(
    time: Res<Time>,
    mut point_light_query: Query<&mut Transform, With<MovingLights>>,
) {
    for mut transform in &mut point_light_query {
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() / 12.0);
    }
}
