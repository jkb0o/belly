use bevy::prelude::*;
use bevy_elements::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
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
