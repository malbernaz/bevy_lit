# Changelog

## 0.10.0

### Features

- **Bevy 0.18 support** – upgraded to the latest Bevy release

## 0.9.1

### Fix

- Fix Voronoi pipeline caching

## 0.9.0

### Fix

- Fix compatibility with bevy 0.17.3

## 0.8.0

### Features

- **Bevy 0.17 support** – upgraded to the latest Bevy release
- **New light sources** – added `SpotLight2d` and `TextureLight2d` for more flexible lighting options
- **Custom 2D light sources** – introduced `Light2dMaterial` and `CustomLight2dPlugin`, enabling creation of entirely new 2D light types

### Breaking

- To match other light sources `AmbientLight2d.brightness` is now `AmbientLight2d.intensity`
- `PointLight2d.radius` is now `PointLight2d.outer_radius`
- `PointLight.shadows_enabled` is now `PointLight2d.cast_shadows`

### Migration

```diff
 commands.spawn((
     ..
     AmbientLight2d {
-         brightness: 0.5,
+         intensity: 0.5,
          ..default()
     }
 ));

 commands.spawn((
     PointLight2d {
-         shadows_enabled: false,
+         cast_shadows: false,
-         radius: 100.0,
+         outer_radius: 100.0,
          ..default()
     }
 ));
```

## 0.7.0

### Features

* **Bevy 0.16 support** – upgraded to the latest Bevy release
* **Light map downsampling** – added a new `LightingSettings2d.scale` option that enables downsampling the light map texture
* **Light penetration** – simulate light bleeding, with configurable intensity and falloff
* **Edge highlighting** – optional visual effect that emphasizes light boundaries 

### Breaking

* `LightingSettings2d.blur` is now a `u32` instead of a float
* Blur is now calculated in **physical pixel space** rather than logical pixels
* Removed the `LightingSettings2d.fixed_resolution` option. (This was previously used to force physical-pixel–based blur; physical pixels are now always used for consistency across resolutions and devices)

### Migration

```rust
commands.spawn((
    Camera2d,
    Lighting2dSettings {
        scale: 0.5, // downsample factor for light map
        edge_intensity: 2.0, // edge highlighting strength
        penetration: PenetrationSettings {
            max: 20.0,          // maximum penetration distance in pixels
            intensity: 1.0,     // brightness factor of penetration light
            falloff: 1.0,       // attenuation curve
            sample_directions: 16, // number of ray directions
            sample_steps: 8,    // steps per ray
        },
        ..default()
    },
));
```

## 0.6.0 - Mesh and Texture Occluders 🎉

### Features

- New `LightOccluder2D` implementation allows any `Mesh2D` or **texture** to be used as an occluder ⚡️
- Better performance! The SDF rendering system has been completely rewritten using Bevy's `Mesh2d` implementation and the **Jump Flood** algorithm
- Adds `tint_occluder` to `LightingSettings2d` that determines whether light occlusion areas should be tinted by the ambient light

### Breaking

- Removed `PointLight2dBundle` and `LightOccluder2dBundle`

### Migration

The basic gist is that now `LightOccluder2d` works like a material for `Mesh2d`.

```rust
commands.spawn((Mesh2d(..), LightOccluder2d::default()));
```

You can also pass a `occluder_mask` to the occluder (any image with a transparent background). The alpha channel will be used to determine wether the pixel should be occluded or not:

```rust
commands.spawn((
    Mesh2d(mesh_handle),
    LightOccluder2d::new(image_handle),
));
```

Note that the same image used for rendering can also be used as the occlusion mask:


```rust
commands.spawn((
    Sprite { image: image_handle.clone(), ..default() },
    Mesh2d(mesh_handle),
    LightOccluder2d::new(image_handle),
));
```

## 0.5.0

### Features

- Introduced **frustum culling** for lighting artifacts 🚀
- Added `shadows_enabled` to `PointLight2d` for shadow projection control
- Added new `stress_test` example

### Breaking Changes

- Removed support for WebGL2

### Migration

```diff
  commands.spawn((
      CursorLight,
      PointLight2d {
          intensity: 4.0,
          radius: 400.0,
          falloff: 3.0,
          color: Color::srgb(1.0, 1.0, 0.0),
+         // defaults to true
+         shadows_enabled: false,
      },
  ));
```

## 0.4.0

### Features

- Bevy 0.15 🎉
- Deprecated `PointLight2dBundle` and `LightOccluder2dBundle` in favor of required components for `PointLight2d` and `LightOccluder2d`

### Migration

```diff
- commands.spawn(PointLight2dBundle {..});
+ commands.spawn(PointLight2d {..});
- commands.spawn(LightOccluder2dBundle {..});
+ commands.spawn(LightOccluder2d {..});
```

## 0.3.0

### Features

- Real soft shadows (the blur effect still available, but the shadow softness implementation doesn't depend on it anymore)
- Raymarch settings configuration

### Migration

```diff
  Lighting2dSettings {
-    shadow_softness: 32.0,
+    blur: 32.0,
+    raymarch: RaymarchSettings::default(),
     ..default()
  }
```

## 0.2.2

### Fixes

- Fixes last release `AmbientLight2d` regression

## 0.2.1

### Fixes

- `Lighting2dSettings` is now mandatory for the lighting to take effect in a given camera
- Fixes `AmbientLight2d` not working when `shadow_softness` is set to 0

## 0.2.0

### Features

- Adds WebGL2 support 🎉
- `AmbientLight2d` and `Lighting2dSettings` are now camera components
- Basic documentation

### Migration

```diff
// Plugin declaration

- App::new().add_plugins((
-     DefaultPlugins,
-     Lighting2dPlugin {
-         ambient_light: AmbientLight2d {
-             brightness: 0.2,
-             color: Color::Srgba(Srgba::hex("#C09AFE").unwrap()),
-         },
-         shadow_softness: 32.0,
-     },
- ));
+ App::new().add_plugins((DefaultPlugins, Lighting2dPlugin));

// Camera setup

- commands.spawn(Camera2dBundle::default());
+ commands.spawn((
+     Camera2dBundle::default(),
+     AmbientLight2d {
+         brightness: 0.2,
+         color: Color::Srgba(Srgba::hex("#C09AFE").unwrap()),
+     },
+     Lighting2dSettings {
+         shadow_softness: 32.0,
+         ..default()
+     },
+ ));
```
