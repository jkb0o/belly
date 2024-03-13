// examples/connections.rs
// cargo run --example connections
//! This example demonstrates how you can connect funcs/handlers
//! to events without writing handler systems. The key feature
//! this example demonstrates is how to connect any Event to
//! any function/handler, so there is no any `eml!` used.
use std::hash::{Hash, Hasher};

use belly::prelude::*;
// use belly_core::relations::connect::WorldEventFilterFunc;
use bevy::{ecs::event::Event, input::keyboard::KeyboardInput, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_event::<ButtonEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, emit_button_events)
        .add_systems(Update, update_counter)
        .run();
}

#[derive(Component, Default)]
pub struct Counter(usize);

#[derive(Event)]
enum ButtonEvent {
    Press(Entity),
    Hover(Entity),
}

// This function acts like Event filter wor world events:
// events that doesn't relate to any entity.
// It is possible to connect to this events with
// commands.connect().event(space_key_released)
fn space_key_released(event: &KeyboardInput) -> bool {
    if event.key_code == Some(KeyCode::Space) {
        match event.state {
            bevy::input::ButtonState::Released => true,
            _ => false,
        }
    } else {
        false
    }
}

// This functions acts like Event filter wor entity events:
// events that may relate to entities. It takes an &Event as
// input and returns EventsSource: zero or more associated entities.
// For each entity in EventSource the func/handler will be executed.
// It is possible to connect to this events with
// commands.connect().entity(entity).on(button_pressed)
fn button_pressed(event: &ButtonEvent) -> EventSource {
    match event {
        ButtonEvent::Press(e) => EventSource::single(*e),
        _ => EventSource::None,
    }
}

fn button_hovered(event: &ButtonEvent) -> EventSource {
    match event {
        ButtonEvent::Hover(e) => EventSource::single(*e),
        _ => EventSource::None,
    }
}

// WorldEventFilterFunc
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    // add_root() function adds some basic nodes and returns
    // root and counter entities: root is the container where
    // the buttons will be spawned and the counter is the
    // entity with the text indicating how many buttons have been
    // removed.
    let (root, counter) = commands.add_root(&asset_server);

    // add button when space released
    commands
        .connect()
        .event(space_key_released)
        .to_func(move |ctx| {
            let btn = ctx.add_button_to(root);
            // remove button when it pressed
            ctx.connect()
                .entity(btn)
                .on(button_pressed)
                // arguments after context in run! macro are passed
                // to the Query prepared on the target entity. By default
                // the target is the same entity the connection was
                // created for (btn in this case).   ----------------------,
                .handle(run!(|ctx, e: Entity| {
                    // |
                    ctx.commands().entity(*e).despawn_recursive(); // |
                })); // |
                     // log button name when it hovered                          // |
            ctx.connect() // |
                .entity(btn) // |
                .on(button_hovered) // |
                // So you can query (and modify) any components   <--------`
                // on the target entity.
                .handle(run!(|_, name: &Name| {
                    info!("{} hovered", name);
                }));
            // You can also provide custom target for connection ------,
            // |
            // track number of removed buttons                          // |
            ctx.connect() // |
                .entity(btn) // |
                .on(button_pressed) // |
                // Like this:               <------------------------------`
                // run! macro in form run!(for entity |...| { })
                // specifies the custom target entity the handler
                // will be executed on.
                //
                // The context (first argument) is optional and may
                // be omited within the run! macro.
                .handle(run!(for counter |counter: &mut Counter| {
                    counter.0 += 1;
                }));
        });
}

fn emit_button_events(
    interactions: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut events: EventWriter<ButtonEvent>,
) {
    for (entity, interaction) in interactions.iter() {
        match interaction {
            Interaction::Pressed => events.send(ButtonEvent::Press(entity)),
            Interaction::Hovered => events.send(ButtonEvent::Hover(entity)),
            _ => {}
        }
    }
}

fn update_counter(mut counters: Query<(&Counter, &mut Text)>) {
    for (counter, mut text) in counters.iter_mut() {
        text.sections[0].value = format!("{}", counter.0);
    }
}

trait SuperCommands {
    fn add_root(&mut self, asset_server: &Res<AssetServer>) -> (Entity, Entity);
}

impl<'w, 's> SuperCommands for Commands<'w, 's> {
    fn add_root(&mut self, asset_server: &Res<AssetServer>) -> (Entity, Entity) {
        let mut root = None;
        let mut counter = None;
        self.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                // flex_wrap: FlexWrap::Wrap,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Px(80.),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    let font = asset_server.load("FiraMono-Medium.ttf");
                    parent.spawn(TextBundle::from_section(
                    "Press [space] to spawn button, click button to remove it. Buttons removed: ",
                    TextStyle { font: font.clone(), font_size: 28., color: Color::BLACK, }
                ));
                    let counter_node = parent
                        .spawn(TextBundle::from_section(
                            "0",
                            TextStyle {
                                font: font.clone(),
                                font_size: 28.,
                                color: Color::BLACK,
                            },
                        ))
                        .insert(Counter::default())
                        .id();
                    counter = Some(counter_node);
                });
            let root_node = parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                ..default()
            });
            root = Some(root_node.id());
        });
        (root.unwrap(), counter.unwrap())
    }
}

trait SuperContext {
    fn add_button_to(&mut self, parent: Entity) -> Entity;
}
impl<'a, 'w, 's, E: Event> SuperContext for EventContext<'a, 'w, 's, E> {
    fn add_button_to(&mut self, parent: Entity) -> Entity {
        let btn = self.commands().spawn_empty();
        let btnid = btn.id();
        let name = pick_name_for(btnid);
        let font = self.load("FiraMono-Medium.ttf".into());
        self.commands().entity(parent).add_child(btnid);
        self.commands()
            .entity(btnid)
            .insert(Name::new(name.to_string()))
            .insert(ButtonBundle {
                background_color: Color::WHITE.into(),
                style: Style {
                    margin: UiRect::all(Val::Px(20.)),
                    width: Val::Auto,
                    height: Val::Px(80.),
                    ..default()
                },
                ..default()
            })
            .with_children(|btn| {
                btn.spawn(TextBundle {
                    text: Text::from_section(
                        name,
                        TextStyle {
                            font,
                            font_size: 28.,
                            color: Color::BLACK,
                        },
                    ),
                    style: Style {
                        margin: UiRect::all(Val::Px(25.)),
                        ..default()
                    },
                    ..default()
                });
            });
        btnid
    }
}

const NAMES: &[&'static str] = &[
    "Peter",
    "James",
    "John",
    "Andrew",
    "Philip",
    "Judas Iscariot",
    "Matthew",
    "Thomas",
    "James, the son of Alpheus",
    "Bartholomew",
    "Judas Thaddeus",
    "Simon Zelotes",
];

pub fn pick_name_for(entity: Entity) -> &'static str {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    entity.hash(&mut hasher);
    let hash = hasher.finish();
    NAMES[(hash % (NAMES.len() as u64)) as usize]
}
