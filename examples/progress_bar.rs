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
            "First five seconds progress:"<br/>
            <progressbar s:width="400px" maximum=3. bind:value=from!(Time:elapsed_seconds())/>
            <br/>
            <progressbar s:height="400px" mode="vertical" maximum=3. bind:value=from!(Time:elapsed_seconds())/>
        </body>
    });
}
