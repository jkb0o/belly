use bevy::prelude::*;
use bevy_elements::build::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(StyleSheet::parse(
        r#"
        .text-input-value {
            color: #2f2f2f;
        }
        .text-input-border:focus {
            background-color: #2f2f2f;
        }
        .center-left {
            align-content: space-around;
            align-items: center;
        }
    "#,
    ));
    let input = commands.spawn_empty().id();
    commands.add(eml! {
        <body s:padding="20px">
            <div c:center-left>
                <TextInput {input} value="world" s:margin-right="10px" s:width="100px"/>
                "Hello, "{bind!(<=input, TextInput.value)}"!"
            </div>
        </body>
    });
}
