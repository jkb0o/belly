use bevy::ecs::query::WorldQuery;
use bevy::ecs::system::SystemParam;
use bevy::utils::{HashMap, HashSet};
use smallvec::SmallVec;
use std::ops::Deref;
use std::ops::DerefMut;
#[cfg(feature = "stylebox")]
use bevy_stylebox::Stylebox;

use crate::ess::PropertyValue;
use crate::tags;
use crate::tags::*;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct ElementBundle {
    pub element: Element,
    #[cfg(feature = "stylebox")]
    pub stylebox: Stylebox,
    #[bundle]
    pub node: NodeBundle,
}

impl Default for ElementBundle {
    fn default() -> Self {
        ElementBundle {
            element: Default::default(),
            #[cfg(feature = "stylebox")]
            stylebox: Stylebox::default(),
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
}
