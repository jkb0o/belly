use belly::prelude::*;
use bevy::prelude::*;

use belly::widgets::common::Label;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let label = commands.spawn_empty().id();
    commands.add(eml! {
        <body s:padding="50px" s:flex-direction="column" s:justify-content="center" s:align-items="center">
            <label {label}/>
            <slider
                s:width="400px"
                minimum=-2.0 value=1. maximum=12.0
                bind:value=to!(label, Label:value|fmt.v("Slider value: {v:0.2}"))
            />
        </body>
    });
}
