// examples/elements-modification.rs
// cargo run --example elements-modification
use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BellyPlugin)
        .add_event::<ToggleClass>()
        .add_startup_system(setup)
        .add_system(process_events)
        .run();
}

pub struct ToggleClass(&'static str);

fn swow_boxes<T: Signal>(ctx: &mut ConnectionGeneralContext<T>) {
    ctx.select(".box").add_class("hidden");
}
fn hide_boxes<T: Signal>(ctx: &mut ConnectionGeneralContext<T>) {
    ctx.select(".box").remove_class("hidden");
}
fn process_events(mut elements: Elements, mut events: EventReader<ToggleClass>) {
    for event in events.iter() {
        for entity in elements.select(".target").entities() {
            elements.toggle_class(entity, event.0.into())
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(StyleSheet::parse(
        r#"
        .box {
            margin: 10px;
            padding: 10px;
        }
        .red {
            background-color: lightcoral;
        }
        .hidden {
            display: none;
        }
        #container {
            background-color: #3f3f3f;
            width: 400px;
            height: 70px;
        }
    "#,
    ));
    commands.add(eml! {
        <body s:padding="50px">
            <button on:press=connect!(|ctx| ctx.send_event(ToggleClass("red")))>
                "Toggle .red class"
            </button>
            <button on:press=connect!(swow_boxes)>
                "Hide boxes"
            </button>
            <button on:press=connect!(hide_boxes)>
                "Show boxes"
            </button>
            <button on:press=connect!(|ctx| ctx.select("#container").toggle_class("hidden"))>
                "Toggle container visibility"
            </button>
            <button on:press=connect!(|ctx| ctx.select("#container > *").toggle_class("hidden"))>
                "Toggle container children visibility"
            </button>
            <br/>
            <span class="box target">"Target span"</span>
            <span id="container">
                <span class="box target">"Target span"</span>
                <span class="box non-target">"Non-target span"</span>
            </span>
        </body>
    });
}
