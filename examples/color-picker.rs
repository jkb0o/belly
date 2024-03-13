// examples/color-picker.rs
// cargo run --example color-picker
use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .run();
}

const COLORS: &[&str] = &[
    // from https://colorswall.com/palette/105557
    // Red     Pink       Purple     Deep Purple
    "#f44336", "#e81e63", "#9c27b0", "#673ab7",
    // Indigo  Blue       Light Blue Cyan
    "#3f51b5", "#2196f3", "#03a9f4", "#00bcd4",
    // Teal    Green      Light      Green Lime
    "#009688", "#4caf50", "#8bc34a", "#cddc39",
    // Yellow  Amber      Orange     Deep Orange
    "#ffeb3b", "#ffc107", "#ff9800", "#ff5722",
];

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(StyleSheet::load("color-picker.ess"));
    let colorbox = commands.spawn_empty().id();
    commands.add(eml! {
        <body>
            <span c:controls>
                <slider c:red
                    bind:value=to!(colorbox, BackgroundColor:0|r)
                    bind:value=from!(colorbox, BackgroundColor:0.r())
                />
                <slider c:green
                    bind:value=to!(colorbox, BackgroundColor:0|g)
                    bind:value=from!(colorbox, BackgroundColor:0.g())
                />
                <slider c:blue
                    bind:value=to!(colorbox, BackgroundColor:0|b)
                    bind:value=from!(colorbox, BackgroundColor:0.b())
                />
                <slider c:alpha
                    bind:value=to!(colorbox, BackgroundColor:0|a)
                    bind:value=from!(colorbox, BackgroundColor:0.a())
                />
            </span>
            <img c:colorbox-holder src="trbg.png">
                <span {colorbox} c:colorbox s:background-color=managed()
                    on:ready=run!(|c: &mut BackgroundColor| {
                        info!("emiting ready");
                        c.0 = Color::WHITE
                })/>
            </img>
            <span c:colors>
            <for color in = COLORS>
                <button on:press=run!(for colorbox |c: &mut BackgroundColor| { c.0 = Color::from_hex(color) })>
                    <span s:background-color=*color s:width="100%" s:height="100%"/>
                </button>
            </for>
            </span>
        </body>
    });
}
