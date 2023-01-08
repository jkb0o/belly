use bevy::prelude::*;
use bevy_stylebox::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(StyleboxPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
