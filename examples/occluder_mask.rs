use bevy::{
    color::palettes::tailwind::{BLUE_300, BLUE_600, GRAY_200, YELLOW_600},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_lit::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Lighting2dPlugin))
        .insert_resource(ClearColor(Color::from(GRAY_200)))
        .add_systems(Startup, setup)
        .add_systems(Update, update_cursor_light)
        .add_systems(FixedUpdate, update_moving_lights)
        .run();
}

#[derive(Component, Default, Clone)]
struct CursorLight;

#[derive(Component, Default, Clone)]
struct MovingLights;

const X_EXTENT: f32 = 700.;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, assets: Res<AssetServer>) {
    commands.spawn_scene(bsn! {
        Camera2d
        Lighting2dSettings {
            blur: 4,
            edge_intensity: 4.0,
            raymarch: RaymarchSettings { max_steps: 32, jitter_contrib: 0.5, sharpness: 10.0 },
        }
        AmbientLight2d { intensity: 0.1, color: { Color::from(BLUE_300) } }
    });

    let lettering_handle = assets.load("abc.png");
    let sprite_image = lettering_handle.clone();
    let occluder_mask = lettering_handle;

    commands.spawn_scene(bsn! {
        Mesh2d({ meshes.add(Rectangle::new(411., 200.)) })
        Sprite { image: sprite_image, color: { Color::WHITE } }
        LightOccluder2d { occluder_mask: occluder_mask }
    });

    commands.spawn_scene(bsn! {
        MovingLights
        Transform
        Visibility
        Children [
            PointLight2d {
                intensity: 2.0,
                outer_radius: 1100.0,
                falloff: 3.0,
                color: { Color::from(BLUE_600) },
            } Transform::from_xyz(-X_EXTENT + 50. / 2., 0.0, 0.0),
            PointLight2d {
                intensity: 2.0,
                outer_radius: 1100.0,
                falloff: 3.0,
                color: { Color::from(BLUE_600) },
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

fn update_moving_lights(
    time: Res<Time>,
    mut point_light_query: Query<&mut Transform, With<MovingLights>>,
) {
    for mut transform in &mut point_light_query {
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() / 12.0);
    }
}
