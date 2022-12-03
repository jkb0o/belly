use bevy::prelude::*;
use bevy_elements::build::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_startup_system(setup)
        .run();
}

#[derive(Component, Default)]
struct Test;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
