use bevy::utils::HashMap;
use bevy::utils::HashSet;

use bevy::prelude::*;
use crate::tags::*;
use crate::property::*;

#[derive(Default)]
pub enum DisplayElement {
    #[default]
    Block,
    Inline,
    // TODO: deside if it event needed
    // InlineBlock,
}

#[derive(Component, Default)]
pub struct Element {
    pub name: Tag,
    pub id: Option<Tag>,
    pub classes: HashSet<Tag>,
    pub state: HashSet<Tag>,
    pub display: DisplayElement,
    pub content: Option<Entity>,
    pub styles: HashMap<Tag, PropertyValues>,
}

impl Element {
    pub fn invalidate(&mut self) { }
}

