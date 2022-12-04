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
        .vertical {
            flex-direction: column
        }
    "#,
    ));
    let input = commands.spawn_empty().id();
    let label = commands.spawn_empty().id();
    commands.add(eml! {
        <body s:padding="20px">
            <div c:vertical>
                <div>"Hello, "<strong>"fella"</strong>"!"</div>
                <textinput {input} value=bind!(=> label, Label.value) s:margin-right="10px" s:width="100px"/>
                <div>
                    "Bind by to-label:   Hello, "<label {label}/>"!"
                </div>
                <div>
                    "Bind by from-label: Hello, "<label value=bind!(<= input, TextInput.value)/>"!"
                </div>
                <div>
                    "Bind by content:    Hello, "{bind!(<= input, TextInput.value)}"!"
                </div>
            </div>
        </body>
    });
}
