// examples/grid.rs
// cargo run --example grid
use belly::prelude::*;
use bevy::prelude::*;

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: [800., 600.].into(),
                title: "Bevy CSS Grid Layout Example".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.add(ess! {
        body {
            // Use the CSS Grid algorithm for laying out this node
            display: grid;
            // Set the grid to have 2 columns with sizes [min-content, minmax(0, 1fr)]
            // - The first column will size to the size of it's contents
            // - The second column will take up the remaining available space
            grid-template-columns: min-content flex(1.0);
            // Set the grid to have 3 rows with sizes [auto, minmax(0, 1fr), 20px]
            // - The first row will size to the size of it's contents
            // - The second row take up remaining available space (after rows 1 and 3 have both been sized)
            // - The third row will be exactly 20px high
            grid-template-rows: auto flex(1.0) 20px;
            background-color: white;
        }
        .header {
            // Make this node span two grid columns so that it takes up the entire top tow
            grid-column: span 2;
            font: bold;
            font-size: 32px;
            color: black;
            display: grid;
            padding: 6px;
        }
        .main {
            // Use grid layout for this node
            display: grid;
            // Make the height of the node fill its parent
            height: 100%;
            // Make the grid have a 1:1 aspect ratio meaning it will scale as an exact square
            // As the height is set explicitly, this means the width will adjust to match the height
            aspect-ratio: 1.0;
            padding: 24px;
            // Set the grid to have 4 columns all with sizes minmax(0, 1fr)
            // This creates 4 exactly evenly sized columns
            grid-template-columns: repeat(4, flex(1.0));
            // Set the grid to have 4 rows all with sizes minmax(0, 1fr)
            // This creates 4 exactly evenly sized rows
            grid-template-rows: repeat(4, flex(1.0));
            row-gap: 12px;
            column-gap: 12px;
            background-color: #2f2f2f;
        }
        // Note there is no need to specify the position for each grid item. Grid items that are
        // not given an explicit position will be automatically positioned into the next available
        // grid cell. The order in which this is performed can be controlled using the grid_auto_flow
        // style property.
        .cell {
            display: grid;
        }
        .sidebar {
            display: grid;
            background-color: black;
            // Align content towards the start (top) in the vertical axis
            align-items: start;
            // Align content towards the center in the horizontal axis
            justify-items: center;
            padding: 10px;
            // Add an fr track to take up all the available space at the bottom of the column so
            // that the text nodes can be top-aligned. Normally you'd use flexbox for this, but
            // this is the CSS Grid example so we're using grid.
            grid-template-rows: auto auto 1fr;
            row-gap: 10px;
            height: 100%;
        }
        .text-header {
            font: bold;
            font-size: 24px;
        }
        .footer {
            // Make this node span two grid column so that it takes up the entire bottom row
            grid-column: span 2;
        }
    });

    commands.add(eml! {
        <body>
            <span c:header>"Belly ESS Grid Layout Example"</span>
            <span c:main>
                <for color in=COLORS>
                    <span c:cell s:background-color=color/>
                </for>
            </span>
            <span c:sidebar>
                <span c:text-header>"War"</span>
                <span c:text-content>
                    "War never changes.\nThe Romans waged war to gather slaves and wealth. Spain built an empire from its lust for gold and territory. Hitler shaped a battered Germany into an economic superpower.\n\nBut war never changes."
                </span>
                <span/>
            </span>
            <span c:footer></span>
        </body>
    });
}
