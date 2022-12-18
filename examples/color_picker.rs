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
    let colorbox = commands.spawn_empty().id();
    commands.add(eml! {
        <body s:padding="50px">
            <span c:controls>
                <slider c:red value=0.75 bind:value=to!(colorbox, BackgroundColor:0|color:r)/>
                <slider c:green value=1.0 bind:value=to!(colorbox, BackgroundColor:0|color:g)/>
                <slider c:blue value=1.0 bind:value=to!(colorbox, BackgroundColor:0|color:b)/>
                <slider c:alpha value=1.0 bind:value=to!(colorbox, BackgroundColor:0|color:a)/>
            </span>
            <span {colorbox} c:colorbox/>
        </body>
    });
    commands.add(StyleSheet::parse(
        r#"
        .controls {
            width: 350px;
            height: 200px;
            flex-direction: column;
            justify-content: space-around;
            margin-right: 20px;
        }
        .controls slider {
            width: 100%;
        }
        .colorbox {
            width: 200px;
            height: 200px;
        }
        .red .slider-low {
            background-color: #F54C36;
        }
        .green .slider-low {
            background-color: #40B052;
        }
        .blue .slider-low {
            background-color: #69A1F5;
        }
    "#,
    ))
}
