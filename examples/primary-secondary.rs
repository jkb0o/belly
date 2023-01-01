use belly::build::*;
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
    commands.add(StyleSheet::parse(
        r#"
        div { 
            padding: 10px;
        }
        .primary {
            background-color: white;
            color: black;
        }
        .primary * {
            color: black;
        }
        .secondary {
            background-color: black;
            color: white;
        }
        .secondary * {
            color: white;
        }
    "#,
    ));
    commands.add(eml! {
        <body>
            <div c:primary>"bevy primary 1"<div>"bevy primary 1 inner"</div>
                <div c:secondary>"bevy secondary 1"<div>"bevy secondary 1 inner"</div>
                    <div c:primary>"bevy primary 2"<div>"bevy primary 2 inner"</div>
                        <div c:secondary>"bevy secondary 2"<div>"beby secondary 2 inner"</div>
                            <div c:primary>"bevy primary 3"<div>"bevy primary 3 inner"</div>
                                <div c:secondary>"bevy secondary 3"<div>"bevy secondary 3 inner"</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </body>
    });
}
