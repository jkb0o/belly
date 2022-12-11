use crate::common::*;
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_elements_core::*;
use bevy_elements_macro::*;

pub(crate) struct ButtonPlugin;
impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BtnEvent>();
        app.init_resource::<BtnGroups>();
        app.register_widget::<Btn>();
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            handle_input_system
                .after(input::Label::Signals)
                .label(Label::HandleInput),
        );
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            handle_states_system
                .after(Label::HandleInput)
                .label(Label::HadnleStates),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
enum Label {
    HandleInput,
    HadnleStates,
}

pub enum BtnEvent {
    Pressed([Entity; 1]),
    Released([Entity; 1]),
}

impl BtnEvent {
    pub fn pressed(&self) -> bool {
        match self {
            BtnEvent::Pressed(_) => true,
            _ => false,
        }
    }
    pub fn released(&self) -> bool {
        match self {
            BtnEvent::Released(_) => true,
            _ => false,
        }
    }
}

impl Signal for BtnEvent {
    fn sources(&self) -> &[Entity] {
        match self {
            BtnEvent::Pressed(source) => source,
            BtnEvent::Released(source) => source,
        }
    }
}

#[derive(Clone, Default, PartialEq, Debug)]
pub enum BtnMode {
    #[default]
    Press,
    Instant,
    Toggle,
    Group(BtnModeGroup),
}

impl TryFrom<&str> for BtnMode {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "press" => Ok(BtnMode::Press),
            "instant" => Ok(BtnMode::Instant),
            "toggle" => Ok(BtnMode::Toggle),
            group if group.starts_with("group(") && group.ends_with(")") => {
                Ok(BtnMode::Group(BtnModeGroup::try_from(
                    group
                        .strip_prefix("group(")
                        .unwrap()
                        .strip_suffix(")")
                        .unwrap(),
                )?))
            }
            _ => Err(format!("Can't parse `{}` as BtnMode", value)),
        }
    }
}

impl TryFrom<Variant> for BtnMode {
    type Error = String;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::String(s) => BtnMode::try_from(s.as_str()),
            variant => {
                if let Some(value) = variant.take::<BtnMode>() {
                    Ok(value)
                } else {
                    Err("Invalid value for BtnMode".to_string())
                }
            }
        }
    }
}

impl From<BtnMode> for Variant {
    fn from(mode: BtnMode) -> Self {
        Variant::boxed(mode)
    }
}

#[derive(PartialEq, Clone, Hash, Eq, Debug)]
pub enum BtnModeGroup {
    String(String),
    Entity(Entity),
}

impl TryFrom<&str> for BtnModeGroup {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "" => Err("Empty group name".to_string()),
            name => Ok(BtnModeGroup::String(name.to_string())),
        }
    }
}

// #[derive(Component)]
#[derive(Component, Widget)]
#[signal(press, BtnEvent, pressed)]
#[signal(release, BtnEvent, released)]
#[alias(button)]
/// The `<button>` tag defines a clickable button.
/// Inside a `<button>` element you can put text (and tags
/// like `<i>`, `<b>`, `<strong>`, `<br>`, `<img>`, etc.)
pub struct Btn {
    #[param]
    pressed: bool,
    #[param]
    mode: BtnMode,
}

impl WidgetBuilder for Btn {
    fn setup(&mut self, ctx: &mut ElementContext) {
        let content = ctx.content();
        ctx.render(eml! {
            <span c:button interactable>
                <span c:button-shadow s:position-type="absolute"/>
                <span c:button-background>
                    <span c:button-foreground>
                        {content}
                    </span>
                </span>
            </span>
        })
    }

    fn styles() -> &'static str {
        r##"
            .button {
                align-content: center;
                min-width: 40px;
                min-height: 40px;
                margin: 5px;
            }
            button:hover .button-foreground {
                background-color: white;
            }
            button:active .button-background {
                margin: 1px -1px -1px 1px;
            }
            button:pressed .button-background {
                margin: 1px -1px -1px 1px;
            }
            button:pressed .button-foreground {
                background-color: #bfbfbf;
            }
            .button-shadow {
                background-color: #4f4f4fb8;
                top: 1px;
                left: 1px;
                bottom: -1px;
                right: -1px;
            }
            .button-background {
                width: 100%;
                margin: -1px 1px 1px -1px;
                padding: 1px;
                background-color: #2f2f2f;
            }
            .button-foreground {
                width: 100%;
                height: 100%;
                background-color: #dfdfdf;
                color: #2f2f2f;
                justify-content: center;
                align-content: center;
                align-items: center;
            }
            .button-foreground * {
                color: #2f2f2f;
            }
        "##
    }
}

struct BtnGroupState {
    pressed: Entity,
    buttons: HashSet<Entity>,
}

impl BtnGroupState {
    fn single(entity: Entity) -> BtnGroupState {
        let mut buttons = HashSet::default();
        buttons.insert(entity);
        BtnGroupState {
            pressed: entity,
            buttons,
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct BtnGroups(HashMap<BtnModeGroup, BtnGroupState>);

fn handle_input_system(
    mut pointer_events: EventReader<PointerInput>,
    mut button_events: EventWriter<BtnEvent>,
    mut buttons: Query<&mut Btn>,
    mut groups: ResMut<BtnGroups>,
    mut state_changes: Local<HashMap<BtnModeGroup, Entity>>,
) {
    state_changes.clear();

    for event in pointer_events
        .iter()
        .filter(|e| e.up() || e.down() || e.pressed())
    {
        for entity in event.sources() {
            let Ok(mut button) = buttons.get_mut(*entity) else {
                continue;
            };
            match (&button.mode, &event.data) {
                (BtnMode::Instant, PointerInputData::Down { presses: _ })
                | (BtnMode::Press, PointerInputData::Pressed { presses: _ }) => {
                    button_events.send(BtnEvent::Pressed([*entity]));
                }
                (BtnMode::Toggle, PointerInputData::Pressed { presses: _ }) => {
                    if button.pressed {
                        button.pressed = false;
                        button_events.send(BtnEvent::Released([*entity]));
                    } else {
                        button.pressed = true;
                        button_events.send(BtnEvent::Pressed([*entity]));
                    }
                }
                (BtnMode::Group(group), PointerInputData::Pressed { presses: _ }) => {
                    if !button.pressed {
                        state_changes.insert(group.clone(), *entity);
                        button_events.send(BtnEvent::Pressed([*entity]));
                    } else {
                    }
                }
                _ => (),
            }
        }
    }
    for (group, pressed_entity) in state_changes.drain() {
        let state = groups
            .entry(group)
            .or_insert_with(|| BtnGroupState::single(pressed_entity));
        state.pressed = pressed_entity;
        for entity in state.buttons.iter() {
            if let Ok(mut button) = buttons.get_mut(*entity) {
                button.pressed = *entity == pressed_entity;
            }
        }
    }
}

fn handle_states_system(
    mut groups: ResMut<BtnGroups>,
    mut elements: Elements,
    mut buttons: Query<(Entity, &mut Btn), Changed<Btn>>,
    mut drop_pressed: Local<HashSet<Entity>>,
) {
    drop_pressed.clear();
    for (entity, mut btn) in buttons.iter_mut() {
        match &btn.mode {
            BtnMode::Instant => elements.set_state(entity, tags::pressed(), false),
            BtnMode::Press => elements.set_state(entity, tags::pressed(), false),
            _ => elements.set_state(entity, tags::pressed(), btn.pressed),
        }
        if let BtnMode::Group(group) = &btn.mode {
            if let Some(state) = groups.get_mut(group) {
                if !state.buttons.contains(&entity) {
                    if btn.pressed {
                        drop_pressed.extend(state.buttons.iter());
                    }
                    state.buttons.insert(entity);
                }
            } else {
                groups.insert(group.clone(), BtnGroupState::single(entity));
                btn.pressed = true;
                elements.set_state(entity, tags::pressed(), true);
            }
        }
    }
    for entity in drop_pressed.iter() {
        if let Ok((entity, mut btn)) = buttons.get_mut(*entity) {
            btn.pressed = false;
            elements.set_state(entity, tags::pressed(), false);
        }
    }
}
