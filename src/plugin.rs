use bevy::{prelude::*, shader::load_shader_library};

use crate::{
    light::{
        point_light::PointLight2dPlugin, spot_light::SpotLight2dPlugin,
        texture_light::TextureLight2dPlugin,
    },
    post_process::Lighting2dSettingsPlugin,
    render::Light2dRenderPlugin,
    voronoi::Voronoi2dPlugin,
};

/// A plugin for adding 2D lighting in the Bevy engine.
///
/// This plugin sets up and configures the necessary components and systems for 2D lighting,
pub struct Lighting2dPlugin;

impl Plugin for Lighting2dPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "view_transformations.wgsl");
        load_shader_library!(app, "settings_types.wgsl");
        load_shader_library!(app, "wrappers_types.wgsl");

        app.add_plugins((
            Light2dRenderPlugin,
            Lighting2dSettingsPlugin,
            Voronoi2dPlugin,
            PointLight2dPlugin,
            SpotLight2dPlugin,
            TextureLight2dPlugin,
        ));
    }
}
