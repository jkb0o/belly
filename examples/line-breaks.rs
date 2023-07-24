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
        <body>
            "Some words" "Some words" <br/> "Some words"<br/><brl/>"Some words"
            "Some words" "Some words" "Some words" "Some words" "Some words"
            <brl/>
            <brl/>
            "Some words" "Some words" "Some words" "Some words" "Some words"
            "Some words" <br/> "Some words" "Some words" "Some words" "Some words"
            "Some words" "Some words" "Some words" "Some words" "Some words"
            "Some words" "Some words" "Some words" "Some words" "Some words"
        </body>
    });
}
