use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component, Default)]
struct Counter {
    count: i32,
}
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // spawn empty Entity to reference it in binds & widgets
    let counter = commands.spawn(Counter::default()).id();
    commands.add(eml! {
        <body s:justify-content="center" s:align-items="center" s:align-content="center">
            // connect the press signal to closure executed on the Counter context
            <button on:press=run!(for counter |c: &mut Counter| c.count += 1 )>"+"</button>
            <span s:width="150px" s:justify-content="center">
                // bind Counter.count property at counter entity to Label.value property
                <label bind:value=from!(counter, Counter:count|fmt.c("Value: {c}"))/>
            </span>
            <button on:press=run!(for counter |c: &mut Counter| c.count -= 1 )>"-"</button>
        </body>
    })
}
