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
        .group { 
            width: 600px;
            align-content: center;
            align-items: center;
        }
        .group .sliders {
            flex-grow: 1.0;
            justify-content: space-around;
        }
        slider {
            width: 150px;
        }
        buttongroup button {
            flex-grow: 1.0;
        }
        .red .range-low {
            background-color: #F54C36;
        }
        .green .range-low {
            background-color: #40B052;
        }
        .blue .range-low {
            background-color: #69A1F5;
        }
        .header {
            min-width: 90px;
        }
    "#,
    ));
    commands.add(eml! {
        <body>
            <img {img} src="icon.png" mode="fit"/>
            <buttongroup bind:value=to!(img, Img:mode) c:group>
                <span c:header>"Mode:"</span>
                <for mode in = &["fit", "cover", "stretch", "source"]>
                    <button value=mode>{mode}</button>
                </for>
            </buttongroup>
            <buttongroup bind:value=to!(img, Img:src) c:group>
                <span c:header>"Source:"</span>
                <button value="icon.png">"Bevy icon"</button>
                <button value="bevy_logo_light.png">"Bevy logo light"</button>
                <button pressed value="bevy_logo_dark.png">"Bevy logo dark"</button>
            </buttongroup>
            <span c:group>
                <span c:header>"Modulate:"</span>
                <span c:sliders>
                    <slider c:red value=1.0 bind:value=to!(img, Img:modulate|r)/>
                    <slider c:green value=1.0 bind:value=to!(img, Img:modulate|g)/>
                    <slider c:blue value=1.0 bind:value=to!(img, Img:modulate|b)/>
                </span>
            </span>
        </body>
    });
}
