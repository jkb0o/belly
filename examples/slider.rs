use belly::prelude::*;
use bevy::prelude::*;

use belly::widgets::common::Label;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BellyPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let label = commands.spawn_empty().id();
    let slider = commands.spawn_empty().id();
    // commands.add(from!(Time:elapsed_seconds()) >> to!(slider, Slider:value));
    commands.add(eml! {
        <body s:padding="50px">
            <slider {slider}
                // s:width="400px"
                s:height="400px"
                mode="vertical"
                minimum=-2.0
                value=1.
                maximum=12.0
                // bind:value=from!(Time:elapsed_seconds())
                bind:value=to!(label, Label:value|fmt.v("Slider value: {v:0.2}"))
            />
            <br/>
            <label {label}/>
        </body>
    });
}
