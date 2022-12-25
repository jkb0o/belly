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
            <div c:primary>"primary 1"<div>"primary 1 inner"</div>
                <div c:secondary>"secondary 1"<div>"secondary 1 inner"</div>
                    <div c:primary>"primary 2"<div>"primary 2 inner"</div>
                        <div c:secondary>"secondary 2"<div>"secondary 2 inner"</div>
                            <div c:primary>"primary 3"<div>"primary 3 inner"</div>
                                <div c:secondary>"secondary 3"<div>"secondary 3 inner"</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </body>
    });
}
