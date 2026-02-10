#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod light;
mod occlusion;
mod plugin;
mod post_process;
mod render;
mod settings;
mod voronoi;
mod wrappers;

/// `use bevy_lit::prelude::*;` to import common components and plugins
pub mod prelude {
    pub use crate::light::{
        point_light::PointLight2d,
        render::{CustomLight2dPlugin, Light2dMaterial, Light2dShaderRef, Light2dSize},
        spot_light::SpotLight2d,
        texture_light::TextureLight2d,
    };
    pub use crate::occlusion::LightOccluder2d;
    pub use crate::plugin::Lighting2dPlugin;
    pub use crate::settings::{
        AmbientLight2d, Lighting2dSettings, PenetrationSettings, RaymarchSettings,
    };
}
