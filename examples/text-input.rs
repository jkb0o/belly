use belly::prelude::*;
use bevy::prelude::*;

use belly::widgets::common::Label;

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
        .text-input-value {
            color: #2f2f2f;
        }
        .text-input-border:focus {
            background-color: #2f2f2f;
        }
        span {
            width: 250px;
        }
    "#,
    ));
    let input = commands.spawn_empty().id();
    let label = commands.spawn_empty().id();
    commands.add(eml! {
        <body s:padding="50px">
            <span>"Type input some text:"</span>
            <textinput {input} bind:value=to!(label, Label:value | fmt.val("I'm bound to label, {val}!")) s:width="150px"/>
            <brl/>

            <span>"Bind input to label:"</span>
            <label {label}/>
            <br/>

            <span>"Bind label from input:"</span>
            <label bind:value=from!(input, TextInput:value | fmt.val("I'm bound from input, {val}!"))/>
            <br/>

            <span>"Bind content from input:"</span>
            "I'm bound by content, "{from!(input, TextInput:value)}"!"
            <br/>

        </body>
    });
}
