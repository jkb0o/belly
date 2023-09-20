// examples/for-loop.rs
// cargo run --example for-loop
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
    let names = vec!["Alice", "Cart", "François", "Yasha"];
    commands.add(eml! {
        <body s:padding="50px" s:flex-direction="column">
            <for name in=names>
                <div>"My name is "{name}</div>
            </for>
        </body>
    });
}
