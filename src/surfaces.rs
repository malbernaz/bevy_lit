use bevy::{
    prelude::*,
    render::render_resource::Extent3d,
    window::{PrimaryWindow, WindowResized},
};

use super::{resources::Lighting2dSettings, utils::create_lighting_surface};

pub const SURFACE: Handle<Image> = Handle::weak_from_u128(32847394924661);
pub const SDF_SURFACE: Handle<Image> = Handle::weak_from_u128(93473874316479);
pub const BLUR_SURFACE: Handle<Image> = Handle::weak_from_u128(42769847620134);

pub fn setup_surfaces(
    mut images: ResMut<Assets<Image>>,
    mut settings: ResMut<Lighting2dSettings>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().expect("Expected primary window.");

    let scale_factor = window.scale_factor();

    settings.viewport = Vec2::new(
        window.physical_width() as f32 / scale_factor,
        window.physical_height() as f32 / scale_factor,
    )
    .as_uvec2();

    let surface = create_lighting_surface(settings.viewport);

    images.insert(&SURFACE, surface.clone());
    images.insert(&SDF_SURFACE, surface.clone());
    images.insert(&BLUR_SURFACE, surface.clone());
}

pub fn update_surfaces(
    mut images: ResMut<Assets<Image>>,
    mut settings: ResMut<Lighting2dSettings>,
    mut window_resized_evr: EventReader<WindowResized>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    for e in window_resized_evr.read() {
        if window_query.get(e.window).is_err() {
            continue;
        };

        settings.viewport = Vec2::new(e.width, e.height).as_uvec2();

        let size = Extent3d {
            width: settings.viewport.x,
            height: settings.viewport.y,
            ..default()
        };

        let sdf_surface = images
            .get_mut(&SDF_SURFACE)
            .expect("sdf image should exist");

        sdf_surface.resize(size);

        let surface = images
            .get_mut(&SURFACE)
            .expect("surface image should exist");

        surface.resize(size);

        let blur_surface = images
            .get_mut(&BLUR_SURFACE)
            .expect("blur image should exist");

        blur_surface.resize(size);
    }
}
