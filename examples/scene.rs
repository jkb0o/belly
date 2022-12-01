use bevy::prelude::*;
use bevy_elements::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_startup_system(setup)
        .run();
    
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn(Camera2dBundle::default());
    commands.add(StyleSheet::parse(r#"
        * {
            font: default-regular;
            color: #cfcfcf;
            font-size: 22px;
        }
    "#));
    commands.spawn(EmlScene::new(asset_server.load("test.eml")));
}