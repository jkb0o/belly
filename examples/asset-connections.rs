// examples/asset-connections.rs
// cargo run --example asset-connections
use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Tell the asset server to watch for asset changes on disk:
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(BellyPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(EmlScene::new(asset_server.load("asset-connections.eml")));
}
