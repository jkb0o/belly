// examples/counter-signals.rs
// cargo run --example counter-signals
use belly::prelude::*;
use bevy::prelude::*;

use belly::widgets::common::Label;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_label)
        .run();
}

#[derive(Component, Default)]
struct Counter {
    count: i32,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // spawn empty Entity to reference it in connections & widgets
    let counter = commands.spawn_empty().id();
    commands.add(eml! {
        <body s:justify-content="center" s:align-items="center" s:align-content="center">
            // connect the press signal to closure executed on the Counter context
            <button on:press=run!(for counter |c: &mut Counter| c.count += 1)>"+"</button>
            <span s:width="150px" s:justify-content="center">
                // insert label widget with Counter component to the predefined entity
                <label {counter} with=Counter/>
            </span>
            <button on:press=run!(for counter |c: &mut Counter| c.count -= 1)>"-"</button>
        </body>
    })
}

fn update_label(mut query: Query<(&Counter, &mut Label), Changed<Counter>>) {
    for (counter, mut label) in query.iter_mut() {
        label.value = format!("Value: {}", counter.count)
    }
}
