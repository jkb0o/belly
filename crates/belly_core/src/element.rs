use bevy::ecs::query::WorldQuery;
use bevy::ecs::system::{Command, SystemParam};
use bevy::utils::{HashMap, HashSet};
use smallvec::SmallVec;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::eml::build::ElementsBuilder;
use crate::ess::{ElementsBranch, PropertyValue, Selector};
use crate::tags;
use crate::tags::*;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct ElementBundle {
    pub element: Element,
    #[bundle]
    pub node: NodeBundle,
}

impl Default for ElementBundle {
    fn default() -> Self {
        ElementBundle {
            element: Default::default(),
            node: NodeBundle {
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct TextElementBundle {
    pub element: Element,
    pub background_color: BackgroundColor,
    #[bundle]
    pub text: TextBundle,
}

impl Default for TextElementBundle {
    fn default() -> Self {
        TextElementBundle {
            element: Element::inline(),
            background_color: BackgroundColor(Color::NONE),
            text: TextBundle {
                text: Text::from_section("", Default::default()),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct ImageElementBundle {
    pub element: Element,
    #[bundle]
    pub image: ImageBundle,
}

impl Default for ImageElementBundle {
    fn default() -> Self {
        ImageElementBundle {
            element: Element::inline(),
            image: ImageBundle {
                background_color: BackgroundColor(Color::WHITE),
                ..Default::default()
            },
        }
    }
}

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
    pub names: SmallVec<[Tag; 2]>,
    pub aliases: SmallVec<[Tag; 2]>,
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
    pub fn invalidate_entity(entity: Entity) -> impl Command {
        move |world: &mut World| {
            if let Some(mut element) = world.entity_mut(entity).get_mut::<Element>() {
                element.invalidate()
            }
        }
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
    pub(crate) roots: Query<'w, 's, Entity, (With<Element>, Without<Parent>)>,
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) elements: Query<'w, 's, ElementsQuery, ()>,
    pub(crate) children: Query<'w, 's, ChildrenQuery, ()>,
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

    /// Selects entities based on provided `ess` query allowing
    /// to modify multiple elements in chained calls:
    /// ```rust
    /// # use belly_core::prelude::*;
    /// fn system(mut elements: Elements) {
    ///   elements.select("#container > *").add_class("hidden");
    /// }
    /// ```
    /// See [elements-modifications.rs](https://github.com/jkb0o/belly/blob/main/examples/elements-modification.rs)
    /// example and [Selector](https://github.com/jkb0o/belly#selectors)
    /// chapter in readme.
    pub fn select<'e>(&'e mut self, query: &str) -> SelectedElements<'w, 's, 'e> {
        let selector: Selector = query.into();
        let mut result = vec![];
        for root in self.roots.iter() {
            let mut branch = vec![];
            // branch.append(&*entuty);
            self.select_branch(root, &mut branch, &selector, &mut result);
        }
        SelectedElements {
            elements: self,
            entities: result,
        }
    }

    fn select_branch(
        &self,
        entity: Entity,
        element_ptrs: &mut Vec<*const Element>,
        selector: &Selector,
        result: &mut Vec<Entity>,
    ) {
        let Ok(elem) = self.elements.get(entity) else {
            return;
        };
        let elem = &*elem as *const Element;
        element_ptrs.push(elem);
        let mut branch = ElementsBranch::new();
        for e in element_ptrs.iter() {
            branch.append(unsafe { e.as_ref().unwrap() })
        }
        if selector.matches(&branch) {
            result.push(entity);
        }
        if let Ok(children) = self.children.get(entity) {
            for ch in children.children {
                self.select_branch(*ch, element_ptrs, selector, result);
            }
        }
        element_ptrs.pop();
    }

    pub fn set_state(&mut self, entity: Entity, state: Tag, value: bool) {
        if let Ok(mut element) = self.elements.get_mut(entity) {
            if !value && element.state.contains(&state) {
                element.state.remove(&state);
                self.invalidate(entity);
            } else if value && !element.state.contains(&state) {
                element.state.insert(state);
                self.invalidate(entity);
            }
        }
    }

    pub fn add_class(&mut self, entity: Entity, class: Tag) {
        if let Ok(mut element) = self.elements.get_mut(entity) {
            if !element.classes.contains(&class) {
                element.classes.insert(class);
                self.invalidate(entity);
            }
        }
    }

    pub fn remove_class(&mut self, entity: Entity, class: Tag) {
        if let Ok(mut element) = self.elements.get_mut(entity) {
            if element.classes.remove(&class) {
                self.invalidate(entity);
            }
        }
    }

    pub fn toggle_class(&mut self, entity: Entity, class: Tag) {
        if let Ok(mut element) = self.elements.get_mut(entity) {
            if element.classes.contains(&class) {
                element.classes.remove(&class);
            } else {
                element.classes.insert(class);
            }
            self.invalidate(entity);
        }
    }

    pub fn set_id(&mut self, entity: Entity, id: Option<Tag>) {
        if let Ok(mut element) = self.elements.get_mut(entity) {
            if element.id != id {
                element.id = id;
                self.invalidate(entity);
            }
        }
    }

    pub fn add_child(&mut self, entity: Entity, eml: ElementsBuilder) {
        let ch = self.commands.spawn_empty().id();
        self.commands.entity(entity).add_child(ch);
        self.commands.add(eml.with_entity(ch));
    }

    pub fn replace(&mut self, entity: Entity, eml: ElementsBuilder) {
        self.commands.entity(entity).despawn_descendants();
        self.commands.add(eml.with_entity(entity));
    }
}

pub struct SelectedElements<'w, 's, 'e> {
    elements: &'e mut Elements<'w, 's>,
    entities: Vec<Entity>,
}

impl<'w, 's, 'e> SelectedElements<'w, 's, 'e> {
    pub fn entities(self) -> Vec<Entity> {
        self.entities
    }
    pub fn add_class<T: Into<Tag>>(&mut self, class: T) -> &mut Self {
        let class = class.into();
        for entity in self.entities.iter() {
            self.elements.add_class(*entity, class);
        }
        self
    }

    pub fn remove_class<T: Into<Tag>>(&mut self, class: T) -> &mut Self {
        let class = class.into();
        for entity in self.entities.iter() {
            self.elements.remove_class(*entity, class);
        }
        self
    }

    pub fn toggle_class<T: Into<Tag>>(&mut self, class: T) -> &mut Self {
        let class = class.into();
        for entity in self.entities.iter() {
            self.elements.toggle_class(*entity, class);
        }
        self
    }

    pub fn set_state<T: Into<Tag>>(&mut self, state: T, value: bool) -> &mut Self {
        let state = state.into();
        for entity in self.entities.iter() {
            self.elements.set_state(*entity, state, value);
        }
        self
    }

    pub fn remove(self) {
        for entity in self.entities {
            self.elements.commands.entity(entity).despawn_recursive();
        }
    }
}
