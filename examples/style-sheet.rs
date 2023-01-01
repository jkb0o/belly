// examples/style-sheet.rs
// cargo run --example style-sheet
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
    commands.add(StyleSheet::load("stylesheet.ess"));
    commands.add(eml! {
        <body>
            <span>"Black span with padding of 25 px and margin of 5px"</span>
            <div>"White div with 10% margin-left property, 3px padding and bold text"</div>
        </body>
    });
}
