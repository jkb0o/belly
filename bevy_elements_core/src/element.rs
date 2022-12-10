use bevy::ecs::query::WorldQuery;
use bevy::ecs::system::SystemParam;
use bevy::utils::{HashMap, HashSet};
use smallvec::SmallVec;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::property::PropertyValue;
use crate::tags;
use crate::tags::*;
use bevy::prelude::*;

#[derive(Default)]
pub enum DisplayElement {
    #[default]
    Block,
    Inline,
    // TODO: deside if it even needed
    // InlineBlock,
}

#[derive(Component, Default)]
pub struct Element {
    pub names: SmallVec<[Tag; 4]>,
    pub id: Option<Tag>,
    pub classes: HashSet<Tag>,
    pub state: HashSet<Tag>,
    pub display: DisplayElement,
    pub content: Option<Entity>,
    pub styles: HashMap<Tag, PropertyValue>,
}

impl Element {
    pub fn is_virtual(&self) -> bool {
        self.names.len() == 0
    }
    pub fn inline() -> Element {
        Element {
            display: DisplayElement::Inline,
            ..default()
        }
    }
    pub fn invalidate(&mut self) {}
    pub fn focused(&self) -> bool {
        self.state.contains(&tags::focus())
    }

    pub fn hovered(&self) -> bool {
        self.state.contains(&tags::hover())
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct ElementsQuery {
    pub entity: Entity,
    element: &'static mut Element,
}

impl<'w, 's> Deref for Elements<'w, 's> {
    type Target = Query<'w, 's, ElementsQuery, ()>;
    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl<'w, 's> DerefMut for Elements<'w, 's> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elements
    }
}

impl Deref for ElementsQueryReadOnlyItem<'_> {
    type Target = Element;
    fn deref(&self) -> &Self::Target {
        &self.element
    }
}
impl Deref for ElementsQueryItem<'_> {
    type Target = Element;
    fn deref(&self) -> &Self::Target {
        &self.element
    }
}
impl DerefMut for ElementsQueryItem<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.element
    }
}

#[derive(WorldQuery)]
pub struct ChildrenQuery {
    children: &'static Children,
}

#[derive(SystemParam)]
pub struct Elements<'w, 's> {
    roots: Query<'w, 's, Entity, (With<Element>, Without<Parent>)>,
    elements: Query<'w, 's, ElementsQuery, ()>,
    children: Query<'w, 's, ChildrenQuery, ()>,
}

impl<'w, 's> Elements<'w, 's> {
    pub fn invalidate(&mut self, tree: Entity) {
        if let Ok(mut element) = self.elements.get_mut(tree) {
            element.invalidate();
        }
        self.children
            .get(tree)
            .map(|c| c.children.iter().copied().collect::<Vec<_>>())
            .ok()
            .map(|c| c.iter().for_each(|e| self.invalidate(*e)));
    }

    pub fn invalidate_all(&mut self) {
        self.roots
            .iter()
            .collect::<Vec<_>>()
            .iter()
            .for_each(|e| self.invalidate(*e));
    }

    pub fn update<F: FnMut(ElementsQueryItem<'_>)>(&mut self, entity: Entity, mut update: F) {
        if let Ok(element) = self.elements.get_mut(entity) {
            update(element)
        }
    }
}
