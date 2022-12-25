use belly::build::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BellyPlugin)
        .add_startup_system(setup)
        .run();
}

#[derive(Component, Default)]
struct Test;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
