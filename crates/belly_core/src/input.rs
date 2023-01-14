use crate::{element::Element, element::Elements, tags};
use bevy::{
    ecs::query::WorldQuery,
    input::InputSystem,
    prelude::*,
    render::camera::RenderTarget,
    ui::{FocusPolicy, UiStack},
    utils::HashSet,
};

pub(crate) struct ElementsInputPlugin;
impl Plugin for ElementsInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PointerInput>()
            .add_event::<RequestFocus>()
            .init_resource::<Focused>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                pointer_input_system
                    .label(Label::Signals)
                    .after(InputSystem),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                tab_focus_system
                    .label(Label::TabFocus)
                    .after(Label::Signals),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                focus_system.label(Label::Focus).after(Label::TabFocus),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                hover_system.label(Label::Hover).after(Label::Signals),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                active_system.label(Label::Active).after(Label::Signals),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum Label {
    Signals,
    TabFocus,
    Focus,
    Hover,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum PointerInputData {
    Down { presses: u8 },
    Up { presses: u8 },
    Pressed { presses: u8 },
    DragStart,
    Drag { from: Vec<Entity> },
    DragStop,
    Motion,
}

#[derive(Debug, Clone, Reflect)]
pub struct PointerInput {
    pub entities: Vec<Entity>,
    pub pos: Vec2,
    pub delta: Vec2,
    pub data: PointerInputData,
}

impl PointerInput {
    pub fn contains(&self, entity: Entity) -> bool {
        for e in self.entities.iter() {
            if *e == entity {
                return true;
            }
        }
        return false;
    }

    pub fn presses(&self) -> u8 {
        use PointerInputData::*;
        match self.data {
            Down { presses } => presses,
            Up { presses } => presses,
            Pressed { presses } => presses,
            _ => 0,
        }
    }

    pub fn down(&self) -> bool {
        if let PointerInputData::Down { presses: _ } = self.data {
            true
        } else {
            false
        }
    }

    pub fn up(&self) -> bool {
        if let PointerInputData::Up { presses: _ } = self.data {
            true
        } else {
            false
        }
    }

    pub fn pressed(&self) -> bool {
        if let PointerInputData::Pressed { presses: _ } = self.data {
            true
        } else {
            false
        }
    }

    pub fn dragging(&self) -> bool {
        if let PointerInputData::Drag { from: _ } = self.data {
            true
        } else {
            false
        }
    }

    pub fn drag_start(&self) -> bool {
        self.data == PointerInputData::DragStart
    }

    pub fn drag_stop(&self) -> bool {
        self.data == PointerInputData::DragStop
    }

    pub fn is_dragging_from(&self, entity: Entity) -> bool {
        if let PointerInputData::Drag { from } = &self.data {
            for e in from.iter() {
                if *e == entity {
                    return true;
                }
            }
            false
        } else {
            false
        }
    }

    pub fn dragging_from(&self) -> &[Entity] {
        if let PointerInputData::Drag { from } = &self.data {
            from
        } else {
            &[]
        }
    }

    pub fn dragging_over_self(&self) -> bool {
        if let PointerInputData::Drag { from } = &self.data {
            // this is just vec equality
            from.len() == self.entities.len()
                && self.entities.iter().zip(from.iter()).all(|(a, b)| a == b)
        } else {
            false
        }
    }

    pub fn motion(&self) -> bool {
        self.data == PointerInputData::Motion
    }
}

/// Contains entities whose Interaction should be set to None
#[derive(Default)]
pub struct State {
    pressed_entities: Vec<Entity>,
    was_down_at: f32,
    was_down: Vec<Entity>,
    presses: u8,
    dragging_from: Vec<Entity>,
    press_position: Option<Vec2>,
    last_cursor_position: Option<Vec2>,
    drag_accumulator: Vec2,
    dragging: bool,
}

/// Main query for [`ui_focus_system`]
#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct NodeQuery {
    entity: Entity,
    node: &'static Node,
    global_transform: &'static GlobalTransform,
    interaction: Option<&'static mut Interaction>,
    focus_policy: Option<&'static FocusPolicy>,
    calculated_clip: Option<&'static CalculatedClip>,
    computed_visibility: Option<&'static ComputedVisibility>,
}

/// The system that sets Interaction for all UI elements based on the mouse cursor activity
///
/// Entities with a hidden [`ComputedVisibility`] are always treated as released.
pub fn pointer_input_system(
    mut state: Local<State>,
    camera: Query<(&Camera, Option<&UiCameraConfig>)>,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    ui_stack: Res<UiStack>,
    time: Res<Time>,
    mut node_query: Query<NodeQuery>,
    mut events: EventWriter<PointerInput>,
) {
    let up =
        mouse_button_input.just_released(MouseButton::Left) || touches_input.any_just_released();
    let down =
        mouse_button_input.just_pressed(MouseButton::Left) || touches_input.any_just_pressed();

    let is_ui_disabled =
        |camera_ui| matches!(camera_ui, Some(&UiCameraConfig { show_ui: false, .. }));

    let cursor_position = camera
        .iter()
        .filter(|(_, camera_ui)| !is_ui_disabled(*camera_ui))
        .filter_map(|(camera, _)| {
            if let RenderTarget::Window(window_id) = camera.target {
                Some(window_id)
            } else {
                None
            }
        })
        .filter_map(|window_id| windows.get(window_id))
        .filter(|window| window.is_focused())
        .find_map(|window| {
            window.cursor_position().map(|mut cursor_pos| {
                cursor_pos.y = window.height() - cursor_pos.y;
                cursor_pos
            })
        })
        .or_else(|| touches_input.first_pressed_position());

    if down {
        state.press_position = cursor_position;
        state.drag_accumulator = Vec2::ZERO;
    }
    let delta = match (cursor_position, state.last_cursor_position) {
        (Some(c), Some(l)) => c - l,
        _ => Vec2::ZERO,
    };
    state.last_cursor_position = cursor_position;

    // prepare an iterator that contains all the nodes that have the cursor in their rect,
    // from the top node to the bottom one.
    let mut moused_over_nodes = ui_stack
        .uinodes
        .iter()
        // reverse the iterator to traverse the tree from closest nodes to furthest
        .rev()
        .filter_map(|entity| {
            if let Ok(node) = node_query.get_mut(*entity) {
                // Nodes that are not rendered should not be interactable
                if let Some(computed_visibility) = node.computed_visibility {
                    if !computed_visibility.is_visible() {
                        // // Reset their interaction to None to avoid strange stuck state
                        // if let Some(mut interaction) = node.interaction {
                        //     // We cannot simply set the interaction to None, as that will trigger change detection repeatedly
                        //     if *interaction != Interaction::None {
                        //         *interaction = Interaction::None;
                        //     }
                        // }

                        return None;
                    }
                }

                let position = node.global_transform.translation();
                let ui_position = position.truncate();
                let extents = node.node.size() / 2.0;
                let mut min = ui_position - extents;
                let mut max = ui_position + extents;
                if let Some(clip) = node.calculated_clip {
                    min = Vec2::max(min, clip.clip.min);
                    max = Vec2::min(max, clip.clip.max);
                }
                // if the current cursor position is within the bounds of the node, consider it for
                // clicking
                let contains_cursor = if let Some(cursor_position) = cursor_position {
                    (min.x..max.x).contains(&cursor_position.x)
                        && (min.y..max.y).contains(&cursor_position.y)
                } else {
                    false
                };

                if contains_cursor {
                    Some(*entity)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<Entity>>()
        .into_iter();

    let mut down_entities = vec![];
    let mut up_entities = vec![];
    let mut pressed_entities = vec![];
    let mut drag_entities = vec![];
    let mut motion_entities = vec![];
    let mut drag_start_entities = vec![];
    if delta.length_squared() > 0.0 && !state.dragging && !state.pressed_entities.is_empty() {
        state.dragging = true;
        drag_start_entities = state.pressed_entities.clone();
    }
    let send_drag_stop = state.dragging && up;
    let mut drag_stop_entities = vec![];
    if send_drag_stop {
        drag_stop_entities = state.dragging_from.clone();
    }

    // set Clicked or Hovered on top nodes. as soon as a node with a `Block` focus policy is detected,
    // the iteration will stop on it because it "captures" the interaction.
    let mut iter = node_query.iter_many_mut(moused_over_nodes.by_ref());
    while let Some(node) = iter.fetch_next() {
        if node.interaction.is_none() {
            continue;
        }
        if node.focus_policy.is_none() {
            continue;
        }
        let entity = node.entity;

        if down {
            state.pressed_entities.push(entity);
            down_entities.push(entity);
        }
        if up {
            up_entities.push(entity);
            let pressed_entity_idx = state.pressed_entities.iter().position(|e| *e == entity);
            if let Some(pressed_entity_idx) = pressed_entity_idx {
                state.pressed_entities.remove(pressed_entity_idx);
                pressed_entities.push(entity);
            }
        }
        // if send_drag_start {
        //     drag_start_entities.push(entity);
        // }
        if delta != Vec2::ZERO {
            if state.dragging {
                drag_entities.push(entity);
            } else {
                motion_entities.push(entity);
            };
        }
        if send_drag_stop {
            drag_stop_entities.push(entity);
        }

        match node.focus_policy.unwrap() {
            FocusPolicy::Block => {
                break;
            }
            FocusPolicy::Pass => { /* allow the next node to be hovered/clicked */ }
        }
    }

    let Some(pos) = cursor_position else { return };
    if down_entities.len() > 0 {
        if time.elapsed_seconds() - state.was_down_at < 0.3 && down_entities == state.was_down {
            state.presses += 1;
        } else {
            state.presses = 0;
        }
        let presses = state.presses + 1;
        state.was_down = down_entities.clone();
        state.was_down_at = time.elapsed_seconds();
        events.send(PointerInput {
            pos,
            delta,
            entities: down_entities,
            data: PointerInputData::Down { presses },
        });
    }
    if pressed_entities.len() > 0 {
        let presses = state.presses;
        events.send(PointerInput {
            pos,
            delta,
            entities: pressed_entities.clone(),
            data: PointerInputData::Pressed { presses },
        });
    }
    if motion_entities.len() > 0 {
        events.send(PointerInput {
            pos,
            delta,
            entities: motion_entities,
            data: PointerInputData::Motion,
        });
    }
    if drag_start_entities.len() > 0 {
        state.dragging_from = drag_start_entities.clone();
        events.send(PointerInput {
            pos,
            delta,
            entities: drag_start_entities,
            data: PointerInputData::DragStart,
        });
    }
    if drag_entities.len() > 0 && drag_stop_entities.is_empty() {
        events.send(PointerInput {
            pos,
            delta,
            entities: drag_entities,
            data: PointerInputData::Drag {
                from: state.dragging_from.clone(),
            },
        });
    }
    if drag_stop_entities.len() > 0 {
        events.send(PointerInput {
            pos,
            delta,
            entities: drag_stop_entities,
            data: PointerInputData::DragStop,
        });
    }
    if up_entities.len() > 0 {
        let presses = state.presses;
        events.send(PointerInput {
            pos,
            delta,
            entities: up_entities,
            data: PointerInputData::Up { presses },
        });
    }

    if up {
        state.pressed_entities.clear();
        state.dragging_from.clear();
        state.press_position = None;
        state.dragging = false;
    }
}

#[derive(Component)]
pub struct Focus(bool);

#[derive(Resource, Default)]
pub struct Focused(Option<Entity>);

pub struct RequestFocus(Entity);

pub fn focus_system(
    mut focused: ResMut<Focused>,
    // mut elements: Query<(Entity, &mut Element)>,
    mut elements: Elements,
    interactable: Query<Entity, (With<Interaction>, With<Element>)>,
    mut signals: EventReader<PointerInput>,
    mut requests: EventReader<RequestFocus>,
    mut dirty: Local<Vec<Entity>>,
) {
    dirty.clear();
    let mut target_focus = None;
    let mut update_required = false;
    for signal in signals.iter().filter(|s| s.down()) {
        for entity in interactable.iter_many(&signal.entities) {
            update_required = true;
            if target_focus.is_none() {
                target_focus = Some(entity);
            }
        }
    }
    for RequestFocus(entity) in requests.iter() {
        update_required = true;
        target_focus = Some(*entity);
    }

    if update_required && target_focus != focused.0 {
        if let Some(was_focused) = focused.0 {
            if let Ok(mut element) = elements.get_mut(was_focused) {
                element.state.remove(&tags::focus());
                dirty.push(was_focused);
            }
        }
        if let Some(target_focus) = target_focus {
            if let Ok(mut element) = elements.get_mut(target_focus) {
                element.state.insert(tags::focus());
                dirty.push(target_focus);
            }
        }
        focused.0 = target_focus;
    }
    dirty.iter().for_each(|e| elements.invalidate(*e));
}

pub fn hover_system(
    mut events: EventReader<PointerInput>,
    mut elements: Elements,
    mut hovered_entities: Local<HashSet<Entity>>,
    mut dirty: Local<Vec<Entity>>,
) {
    dirty.clear();
    let mut any_motion = false;
    let new_hovered_entities: HashSet<_> = events
        .iter()
        .filter(|e| e.motion() || e.dragging())
        .map(|e| {
            any_motion = true;
            e
        })
        .flat_map(|e| e.entities.iter())
        .map(|e| *e)
        .collect();
    if !any_motion {
        return;
    }

    // remove hovered state
    for entity in hovered_entities.difference(&new_hovered_entities) {
        if let Ok(mut element) = elements.get_mut(*entity) {
            element.state.remove(&tags::hover());
            dirty.push(*entity);
        }
    }
    // add hovered state to newely hovered entityes
    for entity in new_hovered_entities.difference(&hovered_entities) {
        if let Ok(mut element) = elements.get_mut(*entity) {
            element.state.insert(tags::hover());
            dirty.push(*entity);
        }
    }
    *hovered_entities = new_hovered_entities;
    for entity in dirty.iter() {
        elements.invalidate(*entity);
    }
}

pub fn active_system(
    mut elements: Elements,
    mut events: EventReader<PointerInput>,
    mut active_elements: Local<HashSet<Entity>>,
    mut add_active: Local<HashSet<Entity>>,
    mut remove_active: Local<HashSet<Entity>>,
) {
    add_active.clear();
    remove_active.clear();
    for event in events.iter() {
        match &event.data {
            PointerInputData::Drag { from } => {
                if event.dragging_over_self() {
                    add_active.extend(from);
                } else {
                    remove_active.extend(from);
                }
            }
            PointerInputData::Down { presses: _ } => {
                add_active.extend(&event.entities);
            }
            PointerInputData::Up { presses: _ } => {
                remove_active.extend(&event.entities);
                remove_active.extend(&*active_elements);
            }
            _ => (),
        }
    }
    for entity in add_active.iter() {
        if active_elements.contains(entity) {
            continue;
        }
        active_elements.insert(*entity);
        elements.invalidate(*entity);
        if let Ok(mut element) = elements.get_mut(*entity) {
            element.state.insert(tags::active());
        }
    }
    for entity in remove_active.iter() {
        if !active_elements.contains(entity) {
            continue;
        }
        active_elements.remove(entity);
        elements.invalidate(*entity);
        if let Ok(mut element) = elements.get_mut(*entity) {
            element.state.remove(&tags::active());
        }
    }
}

pub fn tab_focus_system(
    keyboard: Res<Input<KeyCode>>,
    elements: Query<(Entity, &Element), With<Interaction>>,
    mut requests: EventWriter<RequestFocus>,
) {
    if !keyboard.just_pressed(KeyCode::Tab) {
        return;
    }
    for (entity, _) in elements.iter() {
        requests.send(RequestFocus(entity));
        break;
    }
}
