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
    let img = commands.spawn_empty().id();
    commands.add(StyleSheet::parse(
        r#"
        body {
            flex-wrap: no-wrap;
            flex-direction: column;
            align-content: center;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }
        img {
            width: 50%;
            height: 70%;
            background-color: grey;
            margin-bottom: 20px;
        }
        buttongroup { 
            width: 600px;
            align-content: center;
            align-items: center;
        }
        buttongroup button {
            flex-grow: 1.0;
        }
    "#,
    ));
    commands.add(eml! {
        <body>
            <img {img} src="icon.png" mode="fit"/>
            <buttongroup bind:value=to!(img, Img:mode)>
                <span s:min-width="80px">"Mode:"</span>
                <button value="fit">"fit"</button>
                <button value="cover">"cover"</button>
                <button value="stretch">"stretch"</button>
                <button value="source">"source"</button>
            </buttongroup>
            <buttongroup bind:value=to!(img, Img:src)>
                <span s:min-width="80px">"Source:"</span>
                <button value="icon.png">"Bevy icon"</button>
                <button value="bevy_logo_light.png">"Bevy logo light"</button>
                <button pressed value="bevy_logo_dark.png">"Bevy logo dark"</button>
            </buttongroup>
        </body>
    });
}
