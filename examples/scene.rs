use belly::prelude::*;
use bevy::{prelude::*, asset::ChangeWatcher};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Tell the asset server to watch for asset changes on disk:
            watch_for_changes: ChangeWatcher::with_delay(std::time::Duration::from_millis(50)),
            ..default()
        }))
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(EmlScene::new(asset_server.load("test.eml")));
}
