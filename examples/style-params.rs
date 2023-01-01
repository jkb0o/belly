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
    let ten_percent = Val::Percent(10.);
    commands.add(eml! {
        <body s:padding="5px">
            <span s:padding="25px" s:margin="5px" s:background-color="black">
                "Black span with padding of 25 px and margin of 5px"
            </span>
            <div s:font="bold" s:padding="3px" s:color="black" s:background-color=Color::WHITE  s:margin-left=ten_percent >
                "White div with 10% margin-left property, 3px padding and bold text"
            </div>
        </body>
    });
}
