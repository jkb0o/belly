use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BellyPlugin)
        .add_event::<MyEvent>()
        .add_startup_system(setup)
        .add_system(greet)
        .add_system(debug_my_event)
        .run();
}

struct MyEvent {
    emited_at: f32,
}

#[derive(Component, Default)]
struct Greet {
    counter: i32,
    // instead of using text message field here with
    // custom (greet) system, we should be able to
    // transform type in bind declaration
    // TODO: link github issue here
    message: String,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.add(eml! {
        <body>
            <row>
                <column s:padding="25px" s:margin="5px" s:background-color="white">
                    <span>"Row 1 Column 1"</span>
                    <button on:press=|ctx| info!("I was pressed at {}", ctx.time().elapsed_seconds())>
                        "1. Press me and look at the logs!"
                    </button>
                </column>
                <column s:padding="25px" s:margin="5px" s:background-color="black">
                    <span>"Row 1 Column 2"</span>
                    <button on:press=|ctx| info!("I was pressed at {}", ctx.time().elapsed_seconds())>
                        "2. Press me and look at the logs!"
                    </button>
                </column>
            </row>
            <row>
                <column s:padding="25px" s:margin="5px" s:background-color="black">
                    <span>"Row 2 Column 1"</span>
                    <button on:press=|ctx| info!("I was pressed at {}", ctx.time().elapsed_seconds())>
                        "3. Press me and look at the logs!"
                    </button>
                </column>
                <column s:padding="25px" s:margin="5px" s:background-color="white">
                    <span>"Row 2 Column 2"</span>
                    <button on:press=|ctx| info!("I was pressed at {}", ctx.time().elapsed_seconds())>
                        "4. Press me and look at the logs!"
                    </button>
                </column>
            </row>
            <row s:background-color="red" s:width="100%">
                <center s:width="fit-content" s:background-color="blue">
                    <span>"Centered text"</span>
                </center>
            </row>
        </body>
    });
    commands.add(StyleSheet::parse(
        r#"
        body: {
            padding: 20px;
            flex-direction: column;
            justify-content: center;
            align-content: center;
            align-items: center;
        }
        .counter {
            max-width: 200px;
            justify-content: space-between;
        }
    "#,
    ));
}

fn greet(mut greets: Query<&mut Greet, Changed<Greet>>) {
    for mut greet in greets.iter_mut() {
        greet.message = format!("Count: {}", greet.counter);
    }
}

fn debug_my_event(mut events: EventReader<MyEvent>) {
    for event in events.iter() {
        info!("MyEvent emitted at {:0.2}", event.emited_at);
    }
}
