The `bevy_stylebox` is plugin for [bevy](https://bevyengine.org/) engine which allows you to fill UI node with sliced by 9 parts region of image:
```rust
// examples/panel.rs
// cargo run --example panel

/// Try to resize window and look how stylebox behaves

use bevy::prelude::*;
use bevy_stylebox::*;

fn main() {
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins)
        .add_plugins(StyleboxPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(StyleboxBundle {
        stylebox: Stylebox {
            slice: UiRect::all(Val::Px(16.)),
            texture: asset_server.load("panel-blue.png"),
            ..default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect::all(Val::Percent(25.)),
            ..default()
        },
        ..default()
    });
}
```

`Stylebox` doesn't add any additional UI components. It renders just like `UiImage`, but generates more vertices in the rendering system.