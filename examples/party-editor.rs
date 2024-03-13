// examples/party-ediotor.rs
// cargo run --example party-editor --release

//! This example demonstrates how to build complex UI by creating
//! custom widgets for custom logic, organize user input with
//! <element on:event=run!(..)> blocks and bind data with
//! <element bind:value=to!(..).
//!
//! There is only single startup `setup` system that populates
//! the window with multimple `Animal` buttons and connects
//! their `press` event to the show-me-the-editor clouser. This
//! closure adds another popup widget, binds it inspector-like inner
//! widgets to the `AnimalState` properties and provide the close button that
//! removes the popup.
//!
//! Each time `Animal` is pressed the new `AnimalEditor` widget is
//! created. It replaces the previous one becouse the `AnimalEditor` itself
//! defined with id="editor" attribute. Widgets with custom ids are unique,
//! so the  previous widget with the same id is removed when the new one
//! is added.
//!
//! > NOTE: In this example a lot if ui entities spawned. With debug (default)
//! target the FPS is really low (see https://github.com/jkb0o/belly/issues/48).
//! Running example with --release flag results in better performance, but it
//! is just hiding the problem.

use belly::build::*;
use belly::widgets::input::button::ButtonWidget;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BellyPlugin)
        .add_systems(Startup, setup)
        .run();
}

/// assets/party-editor folder has 8 subfolders in total,
/// each subfolder consists of images with animals
/// with provided style, so
/// - assets/party-editor/style-0/ contains all animals
///   for the 0 style
/// - assets/party-editor/style-1/ contains all animals
///   for the 1 style
/// and so on.
///
/// NUM_STYLES reuquired for proper build final avatar
/// image source from style + animal and for randomizing
/// the animal with AnimalState.randomize()
const NUM_STYLES: u8 = 8;

/// The `ANIMALS` specifies all possible animals images
/// inside the `assets/party-editor/style-*/ folder
const ANIMASLS: &[&'static str] = &[
    "elephant", "giraffe", "hippo", "monkey", "panda", "parrot", "penguin", "pig", "rabbit",
    "snake",
];

/// The `NAMES` used to picking names for default animals
const NAMES: &[&'static str] = &[
    "Brian",
    "Jimi",
    "Janis",
    "Jim",
    "Jean-Michel",
    "Kurt",
    "Amy",
    "Walkie",
    "Alexander",
    "Dave",
    "Gary",
    "Kim",
    "Amar",
    "Rupert",
    "Pamela",
    "Cecilia",
];

/// Predefined `COLORS` used to proper randomize default `AnimalState`
/// with `Animal` widget and build the animal editor interface
const COLORS: &[&'static str] = &[
    // from https://colorswall.com/palette/105557
    "#f44336", "#e81e63", "#9c27b0", "#673ab7", "#3f51b5", "#2196f3", "#03a9f4", "#00bcd4",
    "#009688", "#4caf50", "#8bc34a", "#cddc39", "#ffeb3b", "#ffc107", "#ff9800", "#ff5722",
];

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(StyleSheet::load("party-editor/styles.ess"));
    commands.add(eml! {
        <body>
            <span id="animals" c:column>
                <span c:row>"Choose & Edit your fighters!"</span>
                <for row in=0..4>
                <span c:row>
                    <for column in=0..4>
                        // `Animal` is created 16 times. The `seed` property
                        // passed to widget to randomize somehow the default appearance
                        // of the widget. The `on:press` specifies what should happen
                        // when the `Animal` got pressed: create new `AnimalEditor`
                        // widget inside the `#popups` container and pass the `animal` entity
                        // and its current data to the `AnimalEditor`
                        <Animal seed = row * 4 + column
                            on:press=run!(|ctx, animal: Entity, character: &AnimalState| {
                                let animal = *animal;
                                let data = character.clone();
                                // Each time the `AnimalEditor` created, the previous
                                // one got destroyed (despawned recursivly). This happens
                                // bocouse `AnimalEditor` widget defined with `editor` id.
                                // The defined `id` attribute of widget makes the widget
                                // unique: only single widget with #id can exists in the world
                                // at the same moment.
                                ctx.select("#popups").add_child(eml! {
                                    <AnimalEditor animal=animal data=data/>
                                });
                            })
                        />
                    </for>
                </span>
                </for>
            </span>
            <span id="popups"/>
        </body>
    });
}

#[widget]
#[extends(ButtonWidget)]
fn Animal(ctx: &mut WidgetContext, ch: &mut AnimalState) {
    let seed = ctx.param("seed".into()).unwrap().take().unwrap();
    ch.randomize(seed);
    let this = ctx.entity();
    let color = ctx.spawn();
    ctx.commands()
        .add(from!(this, AnimalState: color) >> to!(color, BackgroundColor:0));
    ctx.render(eml! {
        <button>
            <span {color} c:animal s:background-color=managed()>
                <img bind:src=from!(this, AnimalState:avatar.image())/>
                <span c:label>
                    <label bind:value=from!(this, AnimalState:name)/>
                </span>
            </span>
        </button>
    })
}

/// The `AnimalStateEdior` widget takes `animal: Entity` and `data: AnimalState` as
/// params and builds the inspecotr-like popup with inner widgets binded
/// to the `AnimalState` properties.
///
/// The `data` struct is required to fulfill the widget with default values
/// (name/avatar/color).
///
/// The `animal` entity is required to bind inner widgets to cortresponding
/// properies, so when you edit the name inside widget, the `Text` component
/// on the corresponding entity is changed automaticly.
///
#[widget]
fn AnimalEditor(ctx: &mut WidgetContext) {
    let Some(animal) = ctx.required_param::<Entity>("animal") else {
        return;
    };
    let Some(data) = ctx.required_param::<AnimalState>("data") else {
        return;
    };
    // The eml! macro expands into somethins like `move |world| { ... }`,
    // so you have to create 'static values to pass them to eml! macro.
    let imgsrc = data.avatar.image().clone();
    let name = data.name.clone();
    let color = data.color.get_hex();
    let background = data.color;
    let avatar = ctx.spawn();
    let this = ctx.this().id();
    ctx.add(from!(animal, AnimalState: color) >> to!(avatar, BackgroundColor:0));
    ctx.render(eml! {
        <span id="editor" c:column>
            <span c:shadow/>
            <span c:border/>
            <span c:popup-header c:editor-buttons>
                <span c:grow/>
                // just despawn current widget when close pressed
                <button on:press=move |c| c.commands().entity(this).despawn_recursive()>"Close"</button>
            </span>
            <span c:column c:content>
                <span>"Name:"</span>
                // 
                <textinput value=name bind:value=to!(animal, AnimalState:name)/>
                <span c:separator/>
                <span>"Avatar:"</span>
                <span {avatar} c:avatar s:background-color=managed() on:ready=run!(|b: &mut BackgroundColor| {
                    b.0 = background;
                })>
                    <img src=imgsrc bind:src=from!(animal, AnimalState:avatar.image())/>
                </span>
                <span c:row c:editor-buttons>
                    <button on:press=run!(for animal |data: &mut AnimalState| {
                        data.avatar.prev_animal();
                    })>"Prev"</button>
                    <span c:grow>"Avatar"</span>
                    <button on:press=run!(for animal |data: &mut AnimalState| {
                        data.avatar.next_animal();
                    })>"Next"</button>
                </span>
                <span c:row c:editor-buttons>
                    <button on:press=run!(for animal |data: &mut AnimalState| {
                        data.avatar.prev_style();
                    })>"Prev"</button>
                    <span c:grow>"Style"</span>
                    <button on:press=run!(for animal |data: &mut AnimalState| {
                        data.avatar.next_style();
                    })>"Next"</button>
                </span>
                <span c:separator/>
                <span>"Color:"</span>
                // buttongroup.source property has type string, so the only way to bind
                // to it is to pass the binding with String-based transofmer. In this case
                // it is `|hex` transformer that changes the color based on the string value.
                <buttongroup id="color-picker" value=color bind:value=to!(animal, AnimalState:color|hex)>
                    <for idx in=0..16>
                        <button value=COLORS[idx]><span s:background-color=COLORS[idx]/></button>
                    </for>
                </buttongroup>
            </span>
        </span>
    });
}

#[derive(Default, Clone)]
struct Avatar {
    style: u8,
    person: u8,
}
impl Avatar {
    pub fn image(&self) -> String {
        format!(
            "party-editor/style-{}/{}.png",
            self.style, ANIMASLS[self.person as usize]
        )
    }
    pub fn next_style(&mut self) {
        self.style += 1;
        if self.style >= NUM_STYLES {
            self.style = 0;
        }
    }
    pub fn prev_style(&mut self) {
        if self.style > 0 {
            self.style -= 1;
        } else {
            self.style = NUM_STYLES - 1;
        }
    }
    pub fn next_animal(&mut self) {
        self.person += 1;
        if self.person >= ANIMASLS.len() as u8 {
            self.person = 0;
        }
    }
    pub fn prev_animal(&mut self) {
        if self.person > 0 {
            self.person -= 1;
        } else {
            self.person = ANIMASLS.len() as u8 - 1;
        }
    }
}

#[derive(Component, Default, Clone)]
/// The AnimalState acts like a model. Changing this model properties
/// affects widgets binded to this model (the background of `Animal`
/// widget is changed as well as background of editor when you edit the
/// animal).
pub struct AnimalState {
    name: String,
    avatar: Avatar,
    color: Color,
}

impl AnimalState {
    pub fn randomize(&mut self, seed: u8) {
        self.name = NAMES[(seed as usize) % NAMES.len()].to_string();
        self.avatar.style = seed % NUM_STYLES;
        self.avatar.person = seed % (ANIMASLS.len() as u8);
        self.color = Color::from_hex(COLORS[(seed as usize) % COLORS.len()]);
    }
}

// this is required to pass AnimalState to widget as param
impl From<AnimalState> for Variant {
    fn from(value: AnimalState) -> Self {
        Variant::boxed(value)
    }
}
