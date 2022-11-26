use std::{ops::Index, iter::Filter};

use bevy::{prelude::*, ecs::query::WorldQuery, ui::{FocusPolicy, UiStack}, render::camera::RenderTarget};

const DRAG_THRESHOLD: f32 = 5.;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum SignalData {
    Down,
    Up,
    Pressed,
    DragStart,
    Drag,
    DragStop,
    Motion,
}

#[derive(Debug)]
pub struct Signal {
    pub entities: Vec<Entity>,
    pub pos: Vec2,
    pub delta: Vec2,
    pub data: SignalData
}

impl Signal {
    pub fn contains(&self, entity: Entity) -> bool {
        for e in self.entities.iter() {
            if *e == entity {
                return true;
            }
        }
        return false;
    }

    pub fn pressed(&self) -> bool {
        self.data == SignalData::Pressed
    }
}


/// Contains entities whose Interaction should be set to None
#[derive(Default)]
pub struct State {
    pressed_entities: Vec<Entity>,
    press_position: Option<Vec2>,
    last_cursor_position: Option<Vec2>,
    drag_accumulator: Vec2,
    dragging: bool
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
pub fn signals_system(
    mut state: Local<State>,
    camera: Query<(&Camera, Option<&UiCameraConfig>)>,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    ui_stack: Res<UiStack>,
    mut node_query: Query<NodeQuery>,
    mut events: EventWriter<Signal>,
) {
    // reset entities that were both clicked and released in the last frame
    // for entity in state.entities_to_reset.drain(..) {
    //     if let Ok(mut interaction) = node_query.get_component_mut::<Interaction>(entity) {
    //         *interaction = Interaction::None;
    //     }
    // }

    let mouse_released =
        mouse_button_input.just_released(MouseButton::Left) || touches_input.any_just_released();

    let mouse_clicked =
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
    
    let mut send_drag_start = false;
    let send_drag_stop = state.dragging && mouse_released;
    if mouse_clicked {
        state.press_position = cursor_position;
        state.drag_accumulator = Vec2::ZERO;
    }
    let delta = match (cursor_position, state.last_cursor_position) {
        (Some(c), Some(l)) => c - l,
        _ => Vec2::ZERO
    };
    state.last_cursor_position = cursor_position;
    if !state.press_position.is_none() && !state.dragging {
        state.drag_accumulator += delta;
        if state.drag_accumulator.length() > DRAG_THRESHOLD {
            send_drag_start = true;
            state.dragging = true;

        }
    }


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
    let mut drag_start_entities = vec![];
    let mut drag_entities = vec![];
    let mut drag_stop_entities = vec![];
    let mut motion_entities = vec![];
    
    // set Clicked or Hovered on top nodes. as soon as a node with a `Block` focus policy is detected,
    // the iteration will stop on it because it "captures" the interaction.
    let mut iter = node_query.iter_many_mut(moused_over_nodes.by_ref());
    while let Some(node) = iter.fetch_next() {
        let entity = node.entity;
        let pos = cursor_position.unwrap();
        let rel = pos - node.global_transform.translation().truncate() + node.node.size() * 0.5;;
        
        if mouse_clicked {
            state.pressed_entities.push(entity);
            down_entities.push(entity);
        }
        if mouse_released {
            up_entities.push(entity);
            let pressed_entity_idx = state.pressed_entities.iter().position(|e| *e == entity);
            if let Some(pressed_entity_idx) = pressed_entity_idx {
                state.pressed_entities.remove(pressed_entity_idx);
                pressed_entities.push(entity);
            }
        }
        if send_drag_start {
            drag_start_entities.push(entity);
        }
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
        
        match node.focus_policy.unwrap_or(&FocusPolicy::Block) {
            FocusPolicy::Block => {
                break;
            }
            FocusPolicy::Pass => { /* allow the next node to be hovered/clicked */ }
        }
    }

    if mouse_released {
        state.pressed_entities.clear();
        state.press_position = None;
        state.dragging = false;
    }
    let Some(pos) = cursor_position else { return };
    if down_entities.len() > 0 {
        events.send(Signal { pos, delta, entities: down_entities, data: SignalData::Down});
    }
    if up_entities.len() > 0 {
        events.send(Signal { pos, delta, entities: up_entities, data: SignalData::Up});
    }
    if pressed_entities.len() > 0 {
        info!("Emitting pressed");
        events.send(Signal { pos, delta, entities: pressed_entities, data: SignalData::Pressed});
    }
    if motion_entities.len() > 0 {
        events.send(Signal { pos, delta, entities: motion_entities, data: SignalData::Motion });
    }
    if drag_start_entities.len() > 0 {
        events.send(Signal { pos, delta, entities: drag_start_entities, data: SignalData::DragStart});
    }
    if drag_entities.len() > 0 {
        events.send(Signal { pos, delta, entities: drag_entities, data: SignalData::Drag });
    }
    if drag_stop_entities.len() > 0 {
        events.send(Signal { pos, delta, entities: drag_stop_entities, data: SignalData::DragStop});
    }

    // reset `Interaction` for the remaining lower nodes to `None`. those are the nodes that remain in
    // `moused_over_nodes` after the previous loop is exited.
    // let mut iter = node_query.iter_many_mut(moused_over_nodes);
    // while let Some(node) = iter.fetch_next() {
    //     if let Some(mut interaction) = node.interaction {
    //         // don't reset clicked nodes because they're handled separately
    //         if *interaction != Interaction::Clicked && *interaction != Interaction::None {
    //             *interaction = Interaction::None;
    //         }
    //     }
    // }
}