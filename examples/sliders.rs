// examples/sliders.rs
// cargo run --example sliders
use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BellyPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(eml! {
        <body s:padding="50px">
            <progressbar s:width="200px" bind:value=from!(Time:elapsed_seconds()*0.2)/>
            <br/>
            <progressbar s:width="200px" bind:value=from!(Time:elapsed_seconds()*0.2)>
                <slot separator>
                    <span s:height="100%" s:min-width="10px" s:background-color="red"/>
                </slot>
            </progressbar>
        </body>
    });
}
