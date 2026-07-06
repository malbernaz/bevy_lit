![bevy_lit demo](https://github.com/malbernaz/bevy_lit/raw/main/static/demo.png)

# `bevy_lit`

A simple 2D lighting library **designed for Bevy**.

## Features

- Multiple light sources including `PointLight2d`, `SpotLight2d` and `TextureLight2d`
- Includes primitives `CustomLight2dPlugin` and `Light2dMaterial` for defining custom light sources
- Light occlusion through `LightOccluder2d` that can be used along side any `Mesh2d`
- Per camera fine grain control over lighting parameters such as shadow softness and more
- Terraria-like light penetration effect
- Web support for WebGPU and WebGL2

## Getting started

### Installation

Install it using the CLI:

```sh
cargo add bevy_lit
```

Or add `bevy_lit` to your `Cargo.lock`:

```toml
[dependencies]
bevy_lit = "*"
```

### Usage

Below is a basic example demonstrating how to set up and use `bevy_lit` in your project:

```rust
use bevy::prelude::*;
use bevy_lit::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Lighting2dPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn_scene(bsn! {
        Camera2d
        Lighting2dSettings
    });

    commands.spawn_scene(bsn! {
        PointLight2d { color: { Color::WHITE }, intensity: 3.0, outer_radius: 200.0, falloff: 2.0 }
    });

    let mesh = meshes.add(Circle::new(50.0));
    commands.spawn_scene(bsn! {
        Mesh2d(mesh)
        LightOccluder2d
        Transform::from_xyz(0.0, 200.0, 0.0)
    });
}
```

## Compatibility

| `bevy`   | `bevy_lit`  |
| -------- | ----------- |
| `0.19`   | `0.11`      |
| `0.18`   | `0.10`      |
| `0.17.3` | `0.9`       |
| `0.17`   | `0.8`       |
| `0.16`   | `0.7`       |
| `0.15`   | `0.4..0.6`  |
| `0.14`   | `0.3`       |

## License

`bevy_lit` is licensed under the MIT License. See [LICENSE](LICENSE) for more details.
