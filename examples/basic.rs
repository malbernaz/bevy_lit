use bevy::{math::vec3, prelude::*, window::PrimaryWindow};
use bevy_lit::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Lighting2dPlugin {
                ambient_light: AmbientLight2d {
                    brightness: 0.2,
                    color: Color::Srgba(Srgba::hex("#C09AFE").unwrap()),
                },
                shadow_softness: 32.0,
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update_cursor_light)
        .add_systems(FixedUpdate, update_moving_lights)
        .run();
}

#[derive(Component)]
struct CursorLight;

#[derive(Component)]
struct MovingLights;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    vec![
        vec3(-150.0, 0.0, 0.0),
        vec3(0.0, -150.0, 0.0),
        vec3(150.0, 0.0, 0.0),
        vec3(0.0, 150.0, 0.0),
    ]
    .into_iter()
    .for_each(|pos| {
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(pos),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(100.0)),
                    ..default()
                },
                ..default()
            },
            LightOccluder2d {
                half_size: Vec2::splat(50.0),
            },
        ));
    });

    commands
        .spawn((MovingLights, SpatialBundle::default()))
        .with_children(|builder| {
            let point_light = PointLight2d {
                intensity: 3.0,
                radius: 1000.0,
                falloff: 3.0,
                ..default()
            };

            builder.spawn(PointLight2dBundle {
                point_light: PointLight2d {
                    color: Color::srgb(0.0, 1.0, 1.0),
                    ..point_light
                },
                transform: Transform::from_xyz(-500.0, 0.0, 0.0),
                ..default()
            });

            builder.spawn(PointLight2dBundle {
                point_light: PointLight2d {
                    color: Color::srgb(1.0, 0.0, 1.0),
                    ..point_light
                },
                transform: Transform::from_xyz(500.0, 0.0, 0.0),
                ..default()
            });
        });

    commands.spawn((
        CursorLight,
        PointLight2dBundle {
            point_light: PointLight2d {
                intensity: 4.0,
                radius: 300.0,
                falloff: 3.0,
                color: Color::srgb(1.0, 1.0, 0.0),
            },
            ..default()
        },
    ));
}

fn update_cursor_light(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut point_light_query: Query<&mut Transform, With<CursorLight>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();
    let mut point_light_transform = point_light_query.single_mut();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate().extend(0.0))
    {
        point_light_transform.translation = world_position;
    }
}

fn update_moving_lights(
    time: Res<Time>,
    mut point_light_query: Query<&mut Transform, With<MovingLights>>,
) {
    for mut transform in &mut point_light_query {
        transform.rotation *= Quat::from_rotation_z(time.delta_seconds() / 4.0);
    }
}
