# Changelog

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
