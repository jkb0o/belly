// examples/stylebox-panel.rs
// cargo run --example stylebox-panel
use belly::prelude::*;
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
        r##"
        span {
            margin: 5px;
            padding: 10px;
            flex-grow: 1;
            color: black;
        }
        .flat {
            stylebox: "circle.basis", 50%, 6px, 0px, #bfbfbf;
        }
        .flat.png {
            stylebox-source: "circle.png";
        }
        .flat.big {
            stylebox-width: 20px;
        }
        .flat.red {
            stylebox-modulate: lightcoral;
        }
        .flat.green {
            stylebox-modulate: lightgreen;
        }
        .flat.blue {
            stylebox-modulate: lightblue;
        }
        .tex {
            stylebox: "panel-grey.png", 24px, 100%, 0px;
        }
        .tex.red {
            stylebox-source: "panel-red.png";
        }
        .tex.green {
            stylebox-source: "panel-green.png";
        }
        .tex.blue {
            stylebox-source: "panel-blue.png";
        }
        
    "##,
    ));
    let styles = &["flat png", "flat basis", "flat big", "tex"];
    let colors = &["red", "green", "blue", "grey"];
    commands.add(eml! {
        <body s:padding="20px">
            "The varios styleboxes. Resize the window to see how it behaves."<br/>
            <for style in=styles>
                <for color in=colors>
                    <span class=style class=color>
                    {color}" "{style}
                    </span>
                </for>
                <br/>
            </for>
            <brl/>
            "The difference between png and basis:"<brl/>
            <img src="png-vs-basis.png" s:width="400px" s:height="400px"/>
        </body>
    });
}
