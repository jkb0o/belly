use belly::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_startup_system(setup)
        .run();
}

const COLORS: &[&'static str] = &[
    // from https://colorswall.com/palette/105557
    "#f44336", // Red
    "#e81e63", // Pink
    "#9c27b0", // Purple
    "#673ab7", // Deep Purple
    "#3f51b5", // Indigo
    "#2196f3", // Blue
    "#03a9f4", // Light Blue
    "#00bcd4", // Cyan
    "#009688", // Teal
    "#4caf50", // Green
    "#8bc34a", // Light Green
    "#cddc39", // Lime
    "#ffeb3b", // Yellow
    "#ffc107", // Amber
    "#ff9800", // Orange
    "#ff5722", // Deep Orange
];

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let colorbox = commands.spawn_empty().id();
    commands.add(eml! {
        <body s:padding="50px">
            <span c:controls on:ready=connect!(colorbox, |c: BackgroundColor| c.0 = Color::WHITE)>
                <slider c:red
                    bind:value=to!(colorbox, BackgroundColor:0|color:r)
                    bind:value=from!(colorbox, BackgroundColor:0.r())
                />
                <slider c:green
                    bind:value=to!(colorbox, BackgroundColor:0|color:g)
                    bind:value=from!(colorbox, BackgroundColor:0.g())
                />
                <slider c:blue
                    bind:value=to!(colorbox, BackgroundColor:0|color:b)
                    bind:value=from!(colorbox, BackgroundColor:0.b())
                />
                <slider c:alpha
                    bind:value=to!(colorbox, BackgroundColor:0|color:a)
                    bind:value=from!(colorbox, BackgroundColor:0.a())
                >
                    <slot grabber>
                        <span s:background-color="green" s:width="100%" s:height="100%"/>
                    </slot>
                </slider>
            </span>
            <img c:colorbox-holder src="trbg.png">
                <span {colorbox} c:colorbox/>
            </img>
            <span c:colors>
            <for color in = COLORS>
                <button on:press=connect!(colorbox, |c: BackgroundColor| { c.0 = Color::from_hex(color) })>
                    <span s:background-color=*color s:width="100%" s:height="100%"/>
                </button>
            </for>
            </span>
        </body>
    });
    commands.add(StyleSheet::parse(
        r#"
        body {
            justify-content: center;
            align-items: center;
        }
        .controls {
            width: 200px;
            height: 200px;
            flex-direction: column;
            justify-content: space-around;
            margin-right: 20px;
        }
        .controls slider {
            width: 100%;
        }
        .colorbox-holder {
            width: 200px;
            height: 200px;
        }
        .colorbox {
            width: 100%;
            height: 100%;
        }
        .colors {
            flex-wrap: wrap;
            width: 200px;
            height: 200px;
            margin-left: 20px;
            justify-content: space-between;
            align-content: space-between;
            padding: -5px;
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
    "#,
    ))
}
