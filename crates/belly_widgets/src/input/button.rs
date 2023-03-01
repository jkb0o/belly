use crate::common::prelude::*;
use crate::tags;
use belly_core::build::*;
use belly_core::input;
use belly_core::relations::connect::EventSource;
use belly_macro::*;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use std::fmt::Debug;
use std::hash::Hash;

pub mod prelude {
    pub use super::Btn;
    pub use super::BtnGroup;
    pub use super::BtnMode;
    pub use super::BtnModeGroup;
    pub use super::BtnModeRepeat;
    pub use super::ButtonWidgetExtension;
    pub use super::ButtongroupWidgetExtension;
}

pub(crate) struct ButtonPlugin;
impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BtnEvent>();
        app.add_event::<ValueChanged<String>>();
        app.init_resource::<BtnGroups>();
        app.register_widget::<ButtonWidget>();
        app.register_widget::<ButtongroupWidget>();
        app.add_system(process_btngroups_system);
        app.add_system(force_btngroups_reconfiguration_system);
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
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            report_btngroup_changes
                .after(Label::HadnleStates)
                .label(Label::ReportChanges),
        );
    }
}

#[widget]
#[signal(value_change: ValueChanged<String> => value_changed)]
/// The current value of the `<buttongroup>`. When you set this property, the
/// corresponding button will become pressed, and all other buttons in the
/// group will have their pressed state removed.
#[param(value: String => BtnGroup:value)]
/// A container for multiple toggle buttons. When a button inside a
/// `<buttongroup>` is clicked, it will toggle its pressed state and emit the
/// `pressed` and `released` signals as appropriate. The `<buttongroup>` will
/// update its own `value` property to match the pressed state of the buttons.
///
/// When you set the `value` property of a `<buttongroup>`, the corresponding
/// button will become pressed, and all other buttons in the group will have
/// their pressed state removed.
fn buttongroup(ctx: &mut WidgetContext) {
    let content = ctx.content();
    ctx.render(eml! {
        <span>{content}</span>
    })
}

#[widget]
#[signal(press:BtnEvent => button_pressed)]
#[signal(release:BtnEvent => button_released)]
/// if button is pressed or not
#[param(pressed:bool => Btn:pressed)]
/// <!-- @inline BtnMode -->
#[param(mode:BtnMode => Btn:mode)]
/// Specifies the `<button>` value passed to parent `<buttongroup>`
/// when this button becomes pressed.
#[param(value:String => Btn:value)]
#[styles = BUTTON_STYLES]
/// The `<button>` tag defines a clickable button.
/// Inside a `<button>` widget you can put text (and tags
/// like `<strong>`, `<br>`, `<img>`, etc.)
/// A button can emit `pressed` and `released` signals.
/// The button behaviour is defined by the `mode` param.
/// When changing its pressed state, button adds `:pressed` ess
/// state to element if it pressed and remove `:pressed` if it releases.
fn button(ctx: &mut WidgetContext) {
    let content = ctx.content();
    let flat = ctx.param("flat".into()).is_some();
    ctx.render(if flat {
        eml! {
            <span c:button interactable>
                {content}
            </span>
        }
    } else {
        eml! {
            <span c:button interactable>
                <span c:button-shadow s:position-type="absolute"/>
                <span c:button-background>
                    <span c:button-foreground>
                        {content}
                    </span>
                </span>
            </span>
        }
    });
}

ess_define! {
    BUTTON_STYLES,
    button {
        align-content: center;
        min-width: 40px;
        min-height: 40px;
        margin: 5px;
    }
    button:hover > span > .button-foreground {
        background-color: white;
    }
    button:active > .button-background {
        margin: 1px -1px -1px 1px;
    }
    button:pressed > .button-background {
        margin: 1px -1px -1px 1px;
    }
    button:pressed > span > .button-foreground {
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
        padding: 5px;
    }
    .button-foreground * {
        color: #2f2f2f;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
enum Label {
    HandleInput,
    HadnleStates,
    ReportChanges,
}

pub enum BtnEvent {
    Pressed(Entity),
    Released(Entity),
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

fn button_pressed(event: &BtnEvent) -> EventSource {
    match event {
        BtnEvent::Pressed(entity) => EventSource::single(*entity),
        _ => EventSource::none(),
    }
}

fn button_released(event: &BtnEvent) -> EventSource {
    match event {
        BtnEvent::Released(entity) => EventSource::single(*entity),
        _ => EventSource::none(),
    }
}

pub struct ValueChanged<T> {
    entity: Entity,
    old_value: T,
    new_value: T,
}

impl<T> ValueChanged<T> {
    pub fn new(entity: Entity, old_value: T, new_value: T) -> ValueChanged<T> {
        ValueChanged {
            entity,
            old_value,
            new_value,
        }
    }
    pub fn old_value(&self) -> &T {
        &self.old_value
    }
    pub fn new_value(&self) -> &T {
        &self.new_value
    }
    pub fn any(&self) -> bool {
        true
    }
}

fn value_changed<T: Send + Sync + 'static>(event: &ValueChanged<T>) -> EventSource {
    EventSource::single(event.entity)
}

pub struct BtnCustom;

#[derive(Clone, Default, PartialEq, Debug)]
/// Specifies the button behavior:
///
/// - `press`: When the button is clicked, it will act as if it was pressed
///   for a single frame after the mouse/touch is released if it was down on
///   the same button. The `released` signal will emit on the same frame,
///   immediately following the `pressed` signal.
///
/// - `instant`: The button will act as if it was pressed immediately after
///   the mouse/touch is clicked, unless it is released first. The button will
///   emit the `pressed` signal immediately after the mouse/touch is clicked,
///   and the `released` signal after the mouse/touch is released.
///
/// - `toggle`: The button will toggle its pressed state on and off. The
///   `pressed` signal will emit when the button is pressed down and the
///   `released` signal will emit when the button is released, unless it is
///   still pressed down, in which case the `pressed` signal will not be
///   emitted.
///
/// - `repeat($speed)`: This mode is similar to `instant`, but the `pressed`
///   signal will also emit periodically based on `$speed`. `$speed` can be
///   a constant value or a sequence of delays between emissions. The following
///   values are accepted:
///   - `fast`, `normal`, and `slow` emit signals starting with some base delay
///     and reduce it over time until the minimum delay is reached.
///   - A sequence in the form `0.5 0.4 0.4 0.25`, with any number of elements,
///     where each element specifies the delay between the previous `pressed`
///     emission and the next one.
///
/// - `group($name)`: Associates the button with a virtual named group. Buttons
///   in the same group will act like toggle buttons, but only one button may
///   have the pressed state at a time.
pub enum BtnMode {
    #[default]
    Press,
    Instant,
    Toggle,
    Repeat(BtnModeRepeat),
    Group(BtnModeGroup),
}

impl TryFrom<&str> for BtnMode {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "press" => Ok(BtnMode::Press),
            "instant" => Ok(BtnMode::Instant),
            "toggle" => Ok(BtnMode::Toggle),
            "repeat" => Ok(BtnMode::Repeat(BtnModeRepeat::default())),
            repeat if repeat.starts_with("repeat(") && repeat.ends_with(")") => {
                Ok(BtnMode::Repeat(BtnModeRepeat::try_from(
                    repeat
                        .strip_prefix("repeat(")
                        .unwrap()
                        .strip_suffix(")")
                        .unwrap(),
                )?))
            }
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

#[derive(PartialEq, Clone, Debug)]
struct FloatSequence(Vec<f32>);

#[derive(PartialEq, Clone, Debug, Deref)]
/// <!-- @type-alias=$repeat -->
pub struct BtnModeRepeat(Vec<f32>);

impl BtnModeRepeat {
    pub fn fast() -> BtnModeRepeat {
        vec![0.5, 0.25, 0.25, 0.1, 0.1, 0.1, 0.05].into()
    }
    pub fn slow() -> BtnModeRepeat {
        vec![0.75, 0.5, 0.55, 0.25].into()
    }

    pub fn normal() -> BtnModeRepeat {
        vec![0.66, 0.33, 0.33, 0.1].into()
    }
}

impl Default for BtnModeRepeat {
    fn default() -> Self {
        BtnModeRepeat::normal()
    }
}

impl Eq for BtnModeRepeat {}
impl Hash for BtnModeRepeat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for item in self.0.iter() {
            state.write(&item.to_le_bytes());
        }
    }
}
impl From<&[f32]> for BtnModeRepeat {
    fn from(values: &[f32]) -> Self {
        BtnModeRepeat(values.iter().cloned().collect())
    }
}

impl From<Vec<f32>> for BtnModeRepeat {
    fn from(values: Vec<f32>) -> Self {
        BtnModeRepeat(values)
    }
}

impl TryFrom<&str> for BtnModeRepeat {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        match value {
            "fast" => Ok(BtnModeRepeat::fast()),
            "slow" => Ok(BtnModeRepeat::slow()),
            "normal" => Ok(BtnModeRepeat::normal()),
            _ => {
                let items: Result<Vec<_>, _> = value
                    .split_whitespace()
                    .map(|v| {
                        v.parse()
                            .map_err(|e| format!("Unable to parse {}: {:?}", v, e))
                    })
                    .collect();
                Ok(BtnModeRepeat(items?))
            }
        }
    }
}

#[derive(Component, Default)]
pub struct Btn {
    pub pressed: bool,
    pub mode: BtnMode,
    pub value: String,
}

#[derive(Component, Default)]
pub struct BtnGroup {
    pub value: String,

    configurated: bool,
}

struct BtnGroupState {
    selected: Entity,
    value: String,
    reported_value: String,
    buttons: HashSet<Entity>,
}

impl BtnGroupState {
    fn single(selected: Entity) -> BtnGroupState {
        let mut buttons = HashSet::default();
        buttons.insert(selected);
        BtnGroupState {
            selected,
            value: "".into(),
            reported_value: "".into(),
            buttons,
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct BtnGroups(HashMap<BtnModeGroup, BtnGroupState>);

#[derive(Default)]
struct RepeatState {
    button: Option<(Entity, BtnModeRepeat)>,
    step: usize,
    seconds_to_hit: f32,
    paused: bool,
}

impl RepeatState {
    fn hits(&mut self, delta: f32) -> Option<Entity> {
        if self.paused || self.button.is_none() {
            return None;
        }
        self.seconds_to_hit -= delta;
        if self.seconds_to_hit > 0. {
            return None;
        }
        let (entity, repeats) = self.button.as_ref().unwrap();
        while self.seconds_to_hit <= 0. {
            let delay = if repeats.is_empty() {
                1.0
            } else {
                repeats[self.step.min(repeats.len() - 1)].abs()
            };
            self.seconds_to_hit += delay;
            self.step += 1;
        }
        Some(*entity)
    }

    fn reset(&mut self) {
        self.button = None;
        self.step = 0;
        self.seconds_to_hit = 0.0;
        self.paused = false;
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn unpause(&mut self) {
        self.paused = false;
    }

    fn is_active(&self) -> bool {
        self.button.is_some()
    }

    fn start(&mut self, btn: Entity, repeat: BtnModeRepeat) {
        self.paused = false;
        self.step = 1;
        self.seconds_to_hit = if repeat.is_empty() { 1.0 } else { repeat[0] };
        self.button = Some((btn, repeat));
    }
}

fn handle_input_system(
    mut pointer_events: EventReader<PointerInput>,
    mut button_events: EventWriter<BtnEvent>,
    mut buttons: Query<&mut Btn>,
    mut groups: ResMut<BtnGroups>,
    mut btn_groups: Query<&mut BtnGroup>,
    mut state_changes: Local<HashMap<BtnModeGroup, (Entity, String)>>,
    mut repeat_state: Local<RepeatState>,
    mut instant_pressed: Local<HashSet<Entity>>,
    time: Res<Time>,
) {
    state_changes.clear();

    if let Some(entity) = repeat_state.hits(time.delta_seconds()) {
        button_events.send(BtnEvent::Pressed(entity));
    }

    for event in pointer_events.iter() {
        for entity in event.entities.iter() {
            if repeat_state.is_active() {
                if event.up() {
                    repeat_state.reset();
                }
                if event.dragging_over_self() {
                    repeat_state.unpause();
                } else if event.dragging() {
                    repeat_state.pause();
                }
            }
            if event.up() {
                for instant in instant_pressed.iter() {
                    if let Ok(mut btn) = buttons.get_mut(*instant) {
                        if btn.pressed {
                            btn.pressed = false;
                        }
                    }
                }
                instant_pressed.clear()
            }

            let Ok(mut btn) = buttons.get_mut(*entity) else {
                continue;
            };

            match (&btn.mode, &event.data) {
                (BtnMode::Instant, PointerInputData::Down { presses: _ }) => {
                    if !btn.pressed {
                        btn.pressed = true;
                    }
                    instant_pressed.insert(*entity);
                    button_events.send(BtnEvent::Pressed(*entity));
                }
                (BtnMode::Instant, PointerInputData::Pressed { presses: _ }) => {
                    if btn.pressed {
                        btn.pressed = false;
                    }
                    button_events.send(BtnEvent::Released(*entity));
                }
                (BtnMode::Press, PointerInputData::Pressed { presses: _ }) => {
                    button_events.send(BtnEvent::Pressed(*entity));
                    button_events.send(BtnEvent::Released(*entity));
                }
                (BtnMode::Repeat(repeat), PointerInputData::Down { presses: _ }) => {
                    repeat_state.start(*entity, repeat.clone());
                    button_events.send(BtnEvent::Pressed(*entity));
                }
                (BtnMode::Repeat(_), PointerInputData::Pressed { presses: _ }) => {
                    button_events.send(BtnEvent::Released(*entity));
                }

                (BtnMode::Toggle, PointerInputData::Pressed { presses: _ }) => {
                    if btn.pressed {
                        btn.pressed = false;
                        button_events.send(BtnEvent::Released(*entity));
                    } else {
                        btn.pressed = true;
                        button_events.send(BtnEvent::Pressed(*entity));
                    }
                }
                (BtnMode::Group(group), PointerInputData::Pressed { presses: _ }) => {
                    if !btn.pressed {
                        state_changes.insert(group.clone(), (*entity, btn.value.clone()));
                        button_events.send(BtnEvent::Pressed(*entity));
                    } else {
                    }
                }
                _ => (),
            }
        }
    }
    for (group, (pressed_entity, pressed_value)) in state_changes.drain() {
        if let BtnModeGroup::Entity(btn_group_id) = &group {
            if let Ok(mut btn_group) = btn_groups.get_mut(*btn_group_id) {
                if let Ok(btn) = buttons.get(pressed_entity) {
                    if btn_group.value != btn.value {
                        btn_group.value = btn.value.clone();
                    }
                }
            }
        }
        let state = groups
            .entry(group)
            .or_insert_with(|| BtnGroupState::single(pressed_entity));
        state.selected = pressed_entity;
        state.value = pressed_value;
        for entity in state.buttons.iter() {
            if let Ok(mut btn) = buttons.get_mut(*entity) {
                let pressed = *entity == pressed_entity;
                if btn.pressed != pressed {
                    btn.pressed = pressed;
                    if !btn.pressed {
                        button_events.send(BtnEvent::Released(*entity));
                    }
                }
            }
        }
    }
}

fn handle_states_system(
    mut groups: ResMut<BtnGroups>,
    mut elements: Elements,
    mut buttons: Query<(Entity, &mut Btn), Changed<Btn>>,
    mut drop_pressed: Local<HashSet<Entity>>,
    mut button_events: EventWriter<BtnEvent>,
) {
    drop_pressed.clear();
    for (entity, mut btn) in buttons.iter_mut() {
        match &btn.mode {
            // BtnMode::Instant => elements.set_state(entity, tags::pressed(), false),
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
                if !btn.pressed {
                    btn.pressed = true;
                    button_events.send(BtnEvent::Pressed(entity));
                }
                elements.set_state(entity, tags::pressed(), true);
            }
        }
    }
    for entity in drop_pressed.iter() {
        if let Ok((entity, mut btn)) = buttons.get_mut(*entity) {
            if btn.pressed {
                btn.pressed = false;
                button_events.send(BtnEvent::Released(entity));
            }
            elements.set_state(entity, tags::pressed(), false);
        }
    }
}

fn process_btngroups_system(
    mut btn_grpups: Query<(Entity, &mut BtnGroup), Changed<BtnGroup>>,
    mut buttons: Query<&mut Btn>,
    mut groups: ResMut<BtnGroups>,
    children: Query<&Children>,
) {
    for (entity, mut group) in btn_grpups.iter_mut() {
        if group.configurated {
            continue;
        }
        group.configurated = true;

        let mode_group = BtnModeGroup::Entity(entity);
        let mode = BtnMode::Group(mode_group.clone());
        let mut default_pressed = None;
        let mut found_state = None;
        let mut pressed_value = None;
        for btnid in find_buttons(entity, &buttons, &children) {
            let state = groups
                .entry(mode_group.clone())
                .or_insert_with(|| BtnGroupState::single(btnid));
            state.buttons.insert(btnid);
            if default_pressed.is_none() {
                default_pressed = Some(btnid);
            }
            if let Ok(mut btn) = buttons.get_mut(btnid) {
                if pressed_value.is_none() && !btn.value.is_empty() {
                    pressed_value = Some(btn.value.clone())
                }
                if btn.mode != mode {
                    btn.mode = mode.clone();
                }
                if !btn.value.is_empty() && btn.value == group.value {
                    default_pressed = Some(btnid);
                    pressed_value = Some(btn.value.clone());
                }
                if group.value.is_empty() && btn.pressed && !btn.value.is_empty() {
                    default_pressed = Some(btnid);
                    pressed_value = Some(btn.value.clone());
                }
            }
            found_state = Some(state)
        }
        if let Some(value) = &pressed_value {
            if &group.value != value {
                group.value = value.clone();
            }
        }
        if let Some(state) = found_state {
            if let Some(btnid) = default_pressed {
                state.selected = btnid;
                if let Some(value) = pressed_value {
                    if state.value != value {
                        state.value = value;
                    }
                }
                if let Ok(mut btn) = buttons.get_mut(btnid) {
                    if !btn.pressed {
                        btn.pressed = true;
                    }
                }
            }
        }
    }
}

fn report_btngroup_changes(
    groups: Query<Entity, Changed<BtnGroup>>,
    mut states: ResMut<BtnGroups>,
    mut events: EventWriter<ValueChanged<String>>,
) {
    for entity in groups.iter() {
        let Some(state) = states.get_mut(&BtnModeGroup::Entity(entity)) else {
            continue
        };
        if state.value != state.reported_value {
            events.send(ValueChanged::new(
                entity,
                state.reported_value.clone(),
                state.value.clone(),
            ));
            state.reported_value = state.value.clone();
        }
    }
}

fn force_btngroups_reconfiguration_system(
    mut btn_gropus: Query<&mut BtnGroup>,
    new_buttons: Query<Entity, Added<Btn>>,
    parents: Query<&Parent>,
) {
    for btnid in new_buttons.iter() {
        for entity in parents.iter_ancestors(btnid) {
            if let Ok(mut group) = btn_gropus.get_mut(entity) {
                group.configurated = false;
            }
        }
    }
}

fn find_buttons(
    entity: Entity,
    buttons: &Query<&mut Btn>,
    children: &Query<&Children>,
) -> Vec<Entity> {
    let mut result = vec![];
    find_buttons_walker(&mut result, entity, buttons, children);
    result
}

fn find_buttons_walker(
    buttons: &mut Vec<Entity>,
    entity: Entity,
    buttons_query: &Query<&mut Btn>,
    children_query: &Query<&Children>,
) {
    if buttons_query.contains(entity) {
        buttons.push(entity)
    }
    if let Ok(children) = children_query.get(entity) {
        for ch in children.iter() {
            find_buttons_walker(buttons, *ch, buttons_query, children_query)
        }
    }
}
