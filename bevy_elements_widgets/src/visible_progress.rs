use std::str::FromStr;

use bevy::{prelude::*, utils::HashMap};
use bevy_elements_core::{eml::build::FromWorldAndParam, *};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LayoutMode {
    Vertical,
    Horizontal,
}

impl From<LayoutMode> for Variant {
    fn from(m: LayoutMode) -> Self {
        Variant::boxed(m)
    }
}

impl FromStr for LayoutMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vertical" => Ok(LayoutMode::Vertical),
            "horizontal" => Ok(LayoutMode::Horizontal),
            s => Err(format!("Don't know how to parse '{s}' as LayoutMode")),
        }
    }
}

impl TryFrom<Variant> for LayoutMode {
    type Error = String;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        value.get_or_parse()
    }
}

impl FromWorldAndParam for LayoutMode {
    fn from_world_and_param(_world: &mut World, param: Variant) -> Self {
        param.get_or(LayoutMode::Horizontal)
    }
}

pub trait VisibleProgress: Component {
    fn get_relative_value(&self) -> f32;
    fn get_low_value_entity(&self) -> Entity;
    fn get_high_value_entity(&self) -> Entity;
    fn get_holder_entity(&self) -> Entity;
    fn get_layout_mode(&self) -> LayoutMode;
    fn progress_updating_locked(&self) -> bool {
        false
    }
}

pub fn update_visible_progress_representation<T: VisibleProgress>(
    progress_components: Query<&T, Or<(Changed<T>, Changed<Node>)>>,
    nodes: Query<&Node>,
    mut styles: Query<&mut Style>,
) {
    for progress in progress_components
        .iter()
        .filter(|s| !s.progress_updating_locked())
    {
        let low_span = progress.get_low_value_entity();
        let high_span = progress.get_high_value_entity();
        let Ok(low) = nodes.get(low_span) else { continue };
        let Ok(high) = nodes.get(high_span) else { continue };
        let Ok(mut style) = styles.get_mut(low_span) else { continue };
        let size = low.size() + high.size();
        let offset = size * progress.get_relative_value();
        match progress.get_layout_mode() {
            LayoutMode::Horizontal => style.min_size.width = Val::Px(offset.x),
            LayoutMode::Vertical => style.min_size.height = Val::Px(offset.y),
        }
    }
}

pub fn configure_visible_progress_layout<T: VisibleProgress>(
    mut elements: Elements,
    progres_components: Query<(Entity, &T), Changed<T>>,
    mut styles: Query<&mut Style>,
    mut configured_modes: Local<HashMap<Entity, LayoutMode>>,
) {
    for (entity, progress) in progres_components.iter() {
        let mode = progress.get_layout_mode();
        if configured_modes.get(&entity) == Some(&mode) {
            continue;
        }
        configured_modes.insert(entity, mode);
        {
            match mode {
                LayoutMode::Horizontal => {
                    elements.set_state(entity, "horizontal".as_tag(), true);
                    elements.set_state(entity, "vertical".as_tag(), false);
                }
                LayoutMode::Vertical => {
                    elements.set_state(entity, "horizontal".as_tag(), false);
                    elements.set_state(entity, "vertical".as_tag(), true);
                }
            }
        }
        {
            let Ok(mut holder) = styles.get_mut(progress.get_holder_entity()) else { continue };
            holder.flex_direction = match mode {
                LayoutMode::Horizontal => FlexDirection::Row,
                LayoutMode::Vertical => FlexDirection::ColumnReverse,
            }
        }
        {
            let Ok(mut low) = styles.get_mut(progress.get_low_value_entity()) else { continue };
            match mode {
                LayoutMode::Horizontal => low.min_size.height = Val::Undefined,
                LayoutMode::Vertical => low.min_size.width = Val::Undefined,
            }
        }
    }
}
