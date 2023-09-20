// examples/selectors.rs
// cargo run --example selectors
use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(StyleSheet::load("selectors.ess"));
    commands.add(eml! {
        <body>
            <button c:red><span c:content>"red"</span></button>
            <button c:green><span c:content>"green"</span></button>
            <button c:blue><span c:content>"blue"</span></button>
        </body>
    });
}
