use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_event::<MyEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, greet)
        .add_systems(Update, debug_my_event)
        .run();
}

#[derive(Event)]
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

#[derive(Component, Default, PartialEq)]
enum ColorBox {
    #[default]
    Red,
    Blue,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let label = commands.spawn_empty().insert(Greet::default()).id();
    let that = commands.spawn_empty().id();
    let colorbox = commands.spawn_empty().insert(ColorBox::Red).id();
    let grow = commands.spawn_empty().id();
    commands.add(eml! {
        <body>
            <div>
                <button on:press=|ctx| info!("I was pressed at {}", ctx.time().elapsed_seconds())>
                    "Press me and look at the logs!"
                </button>
            </div>
            <div>
                <button on:press=|ctx| ctx.send_event(MyEvent { emited_at: ctx.time().elapsed_seconds() })>

                    "I will send custom event, check the logs"
                </button>
            </div>
            <div>
                <button on:press=run!(|ctx, source: Entity| ctx.entity(*source).despawn_recursive()) >
                    "I will disappear"
                </button>
            </div>
            <div c:column>
                <button c:bluex on:press=run!(for that |ctx, e: Entity| ctx.commands().entity(*e).despawn_recursive() )>
                    "That will disappear:"
                </button>
                <strong {that}>"THAT"</strong>
            </div>
            <div c:counter>
                <button mode="repeat(normal)" on:press=run!(for label |g: &mut Greet| g.counter += 1)>
                    <strong>"+"</strong>
                </button>
                <label {label} bind:value=from!(label, Greet:message)/>
                <button mode="repeat(fast)" on:press=run!(for label |g: &mut Greet| g.counter -= 1)>
                    <strong>"-"</strong>
                </button>
            </div>
            <div>
                <button on:press=run!(for colorbox |ctx, b: &mut ColorBox| {
                    if **b == ColorBox::Red {
                        **b = ColorBox::Blue;
                        ctx.select("#colorbox").add_child(eml! {
                            <div c:blue id="color">"I'm blue"</div>
                        });
                    } else {
                        **b = ColorBox::Red;
                        ctx.select("#colorbox").add_child(eml! {
                            <div c:red id="color">"I'm red"</div>
                        });
                    }
                })>
                    <div c:colorbox {colorbox} id="colorbox">
                        <div id="color" c:red>"I'm red"</div>
                    </div>
                </button>
            </div>
            <div>
                <button {grow} s:width=managed() on:press=run!(for grow |s: &mut Style| {
                    s.width = Val::Px(if let Val::Px(width) = s.width {
                        width + 5.
                    } else {
                        205.
                    });
                })>
                    "I can grow!"
                </button>
            </div>
        </body>
    });
    commands.add(StyleSheet::parse(
        r#"
        body {
            flex-direction: column;
            padding: 20px;
            justify-content: center;
            align-content: center;
            align-items: center;
        }
        body > div {
            justify-content: center;
            align-content: center;
            align-items: center;
        }
        .counter {
            max-width: 200px;
            justify-content: space-between;
        }
        
        orange.button {
            min-width: 200px;
        }
        .colorbox {
            width: 200px;
            height: 175px;
        }
        .colorbox > div {
            width: 100%;
            height: 100%;
            justify-content: center;
            align-items: center;
        }
        .red {
            background-color: indianred;
            color: lightblue;
        }
        .blue {
            background-color: lightblue;
            color: indianred;
        }
        .blue .button-foreground {
            background-color: lightblue;
            color: indianred;
            padding: 10px;
        }
        .column {
            flex-direction: column;
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
    for event in events.read() {
        info!("MyEvent emitted at {:0.2}", event.emited_at);
    }
}
