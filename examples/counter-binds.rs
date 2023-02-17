use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BellyPlugin)
        .add_startup_system(setup)
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
        <body s:justify-content="center" s:align-items="center">
            // connect the press signal to closure executed on the Counter context
            <button on:press=connect!(counter, |c: Counter| c.count += 1)>"+"</button>
            <span s:width="150px" s:justify-content="center">
                // bind Counter.count property at counter entity to Label.value property
                <label bind:value=from!(counter, Counter:count|fmt.c("Value: {c}"))/>
            </span>
            <button on:press=connect!(counter, |c: Counter| c.count -= 1)>"-"</button>
        </body>
    })
}
