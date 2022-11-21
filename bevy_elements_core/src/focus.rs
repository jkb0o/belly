use bevy::prelude::*;

use crate::{Element, tags};


#[derive(Component)]
pub struct Focus(bool);

#[derive(Default)]
pub struct Focused(Option<Entity>);


pub fn update_focus(
    mut focused: ResMut<Focused>,
    interacted: Query<(Entity, &Interaction), Changed<Interaction>>,
    focusable: Query<(), With<Element>>,
    mut elements: Query<&mut Element>,
    children: Query<&Children>
) {
    let mut target_focus = None;
    let mut update_required = false;
    for (entity, interaction) in interacted.iter() {
        if *interaction == Interaction::Clicked {
            info!("Cliccked: {:?}", entity);
            update_required = true;
            if target_focus.is_none() && focusable.contains(entity) {
                target_focus = Some(entity);
            }
        }
    }
    if update_required && target_focus != focused.0 {
        info!("New focused node: {:?}", target_focus);
        if let Some(was_focused) = focused.0 {
            if let Ok(mut element) = elements.get_mut(was_focused) {
                element.state.remove(&tags::focus());
                invalidate_tree(was_focused, &mut elements, &children);
            }
        }
        if let Some(target_focus) = target_focus {
            if let Ok(mut element) = elements.get_mut(target_focus) {
                element.state.insert(tags::focus());
                invalidate_tree(target_focus, &mut elements, &children);
            }
        }
        focused.0 = target_focus;
    }
}

fn invalidate_tree(
    node: Entity,
    q_elements: &mut Query<&mut Element>,
    q_children: &Query<&Children>
) {
    if let Ok(mut element) = q_elements.get_mut(node) {
        element.invalidate();
    }
    if let Ok(children) = q_children.get(node) {
        for child in children.iter() {
            invalidate_tree(*child, q_elements, q_children);
        }
    }
}