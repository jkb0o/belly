// examples/sliders.rs
// cargo run --example sliders
// TODO: rename to progressbars
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
    commands.add(eml! {
        <body s:padding="50px" s:flex-direction="column">
            <progressbar s:width="200px" bind:value=from!(Time:elapsed_seconds()*0.2)/>
            <progressbar s:width="200px" bind:value=from!(Time:elapsed_seconds()*0.2)>
                <slot separator>
                    <span s:height="100%" s:min-width="10px" s:background-color="red"/>
                </slot>
            </progressbar>
        </body>
    });
}
