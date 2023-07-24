// examples/image-sources.rs
// cargo run --example image-sources
use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let img0 = "icon.png";
    let img1: Handle<Image> = asset_server.load("bevy_logo_light.png");
    let img2: Handle<Image> = asset_server.load("bevy_logo_dark.png");
    commands.add(StyleSheet::parse(
        "
        body { padding: 50px; }
        body > img { width: 150px; height: 150px; margin: 50px; }
    ",
    ));
    commands.add(eml! {
        <body>
            <img src=img0/>
            <img src=img1/>
            <img src=img2/>
        </body>
    });
}
