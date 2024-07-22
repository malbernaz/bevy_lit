# `bevy_lit`

`bevy_lit` is a simple and easy-to-use 2D lighting library for Bevy, designed to work seamlessly with a single camera setup. The library provides basic lighting functionalities through the types: `AmbientLight2d`, `LightOccluder2d`, and `PointLight2d`.

<div alight="center">

![bevy_lit demo](https://github.com/malbernaz/bevy_lit/blob/main/static/demo.png)

</div>

## Features

- **AmbientLight2d**: Provides a general light source that illuminates the entire scene uniformly.
- **LightOccluder2d**: Creates shadows and blocks light from `PointLight2d`.
- **PointLight2d**: Emits light from a specific point, simulating light sources like lamps or torches.

## Disclaimer

Please note that `bevy_lit` is in an early development stage. Users will find bugs and incomplete features. Additionally, the documentation is currently lacking. I appreciate any feedback and contributions to help improve the library.

## Getting Started

### Installation

**Note:** `bevy_lit` is not yet available as a crate on [crates.io](https://crates.io/).

To use it, you can clone the repository directly from GitHub, if you want to fine tune the library to your needs, or add the repo to your `Cargo.lock`:

```toml
[dependencies]
bevy_lit = { git = "https://github.com/malbernaz/bevy_lit" }
```

### Demo

```sh
cargo run --example basic
```

### Usage

Below is a basic example demonstrating how to set up and use `bevy_lit` in your Bevy project:

```rust
use bevy::prelude::*;
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
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn_bundle(PointLight2dBundle {
        point_light: PointLight2d {
            color: Color::rgb(1.0, 1.0, 1.0),
            intensity: 3.0,
            radius: 200.0,
            falloff: 2.0,
        },
        ..default()
    });

    commands.spawn_bundle(LightOccluder2dBundle {
        light_occluder: LightOccluder2d::new(Vec2::new(50.0, 50.0)),
        transform: Transform::from_xyz(0.0, 200.0, 0.0),
        ..default()
    });
}
```

## Implementation

`bevy_lit` uses signed distance fields (SDFs) to compute the occluders' distances. To soften the shadows, a blur is applied. This approach is not ideal and might have limitations in terms of performance and visual accuracy, but it provides a starting point for basic 2D lighting effects.

## Acknowledgement

This library took heavy inspiration from the work of other developers. I learned a lot about lighting and Bevy development by reading the source code of the following crates:

- [`bevy_light_2d`](https://github.com/jgayfer/bevy_light_2d)
- [`bevy-magic-light-2d`](https://github.com/zaycev/bevy-magic-light-2d)

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

`bevy_lit` is licensed under the MIT License. See [LICENSE](LICENSE) for more details.
