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
        <body
            s:padding="50px"
            s:flex-direction="column"
            s:align-items="start"
        >
            "First five seconds progress:"
            <progressbar s:width="400px" maximum=3. bind:value=from!(Time:elapsed_seconds())/>
            <progressbar s:height="400px" mode="vertical" maximum=3. bind:value=from!(Time:elapsed_seconds())/>
        </body>
    });
}
