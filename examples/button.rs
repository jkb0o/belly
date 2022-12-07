use bevy::prelude::*;
use bevy_elements::build::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ElementsPlugin)
        .add_startup_system(setup)
        .add_system(greet)
        .register_widget::<orange>()
        .run();
}

#[widget]
#[extends(button)]
#[style("background-color: darkorange")]
#[style("padding: 5px")]
#[style("color: #2f2f2f")]
#[style("margin: 5px")]
// the rest of the styles should be extended
// from button (now them are copy-pasted)
// TODO: link gihub issue here
#[style("justify-content: space-around")]
#[style("align-content: center")]
#[style("min-width: 40px")]
#[style("min-height: 40px")]
fn orange(ctx: &mut ElementContext) {
    let content = ctx.content();
    ctx.render(eml! {
        <button>{content}</button>
    })
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
    let orange = commands.spawn_empty().id();
    commands.add(eml! {
        <body>
            <div>
                <button on:press=connect!(|ctx| {
                    info!(
                        "I was pressed at {}, pos: {:?}!",
                        ctx.time().elapsed_seconds(),
                        ctx.event().pos
                    )
                })>
                    "Press me and look at the logs!"
                </button>
            </div>
            <div>
                <button on:press=connect!(|ctx| ctx.source().despawn_recursive()) >
                    "I will disappear"
                </button>
            </div>
            <div>
                <button on:press=connect!(that, |ctx| ctx.target().despawn_recursive() )>
                    "That will disappear:"
                </button>
                <strong {that}>"THAT"</strong>
            </div>
            <div c:counter>
                <button on:press=connect!(label, |g: Greet| g.counter += 1)>
                    <strong>"+"</strong>
                </button>
                <label {label} value=bind!(<= label, Greet.message)/>
                <button on:press=connect!(label, |g: Greet| g.counter -= 1)>
                    <strong>"-"</strong>
                </button>
            </div>
            <div>
                <button on:press=connect!(colorbox, |ctx, b:ColorBox| {
                    if **b == ColorBox::Red {
                        **b = ColorBox::Blue;
                        ctx.replace(eml! { <div c:blue>"I'm blue"</div> });
                    } else {
                        **b = ColorBox::Red;
                        ctx.replace(eml! { <div c:red>"I'm red"</div> });
                    }
                })>
                    <div c:colorbox>
                        <div c:red {colorbox}>"I'm red"</div>
                    </div>
                </button>
                <br/>
            </div>
            <div>
                <orange {orange} on:press=connect!(orange, |s: Style| {
                    s.size.width = Val::Px(if let Val::Px(height) = s.size.width {
                        height + 5.
                    } else {
                        205.
                    });
                })>
                    "I can grow!"
                </orange>
            </div>
        </body>
    });
    commands.add(StyleSheet::parse(
        r#"
        body: {
            padding: 20px;
            justify-content: center;
            align-content: center;
            align-items: center;
        }
        div: {
            justify-content: center;
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
            height: 75px;
        }
        .colorbox div {
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
    "#,
    ));
}

fn greet(mut greets: Query<&mut Greet, Changed<Greet>>) {
    for mut greet in greets.iter_mut() {
        greet.message = format!("Count: {}", greet.counter);
    }
}
