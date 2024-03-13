use bevy::ecs::component::Tick;
use bevy::ecs::query::WorldQuery;
use bevy::ecs::system::{Command, CommandQueue, SystemMeta, SystemParam};
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::ui::UiSystem;
use bevy::utils::{HashMap, HashSet};
use smallvec::SmallVec;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::eml::Eml;
use crate::ess::{ElementsBranch, PropertyValue, Selector};
use crate::tags;
use crate::tags::*;
use bevy::prelude::*;

pub struct ElementsPlugin;
impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ElementIdIndex>();
        app.add_systems(
            PostUpdate,
            invalidate_elements
                .in_set(InvalidateElements)
                .before(UiSystem::Layout),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct InvalidateElements;

#[derive(Bundle)]
pub struct ElementBundle {
    pub element: Element,
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
    pub text: TextBundle,
}

impl Default for TextElementBundle {
    fn default() -> Self {
        TextElementBundle {
            element: Element::inline(),
            text: TextBundle {
                text: Text::from_section("", Default::default()),
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct ImageElementBundle {
    pub element: Element,
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
    pub(crate) id: Option<Tag>,
    pub classes: HashSet<Tag>,
    pub state: HashSet<Tag>,
    pub styles: HashMap<Tag, PropertyValue>,
}

impl Element {
    pub fn is_virtual(&self) -> bool {
        self.names.len() == 0
    }
    pub fn inline() -> Element {
        Element { ..default() }
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
            if let Some(mut entity) = world.get_entity_mut(entity) {
                if let Some(mut element) = entity.get_mut::<Element>() {
                    element.invalidate()
                }
            }
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct ElementIdIndex(HashMap<Tag, Entity>);

#[derive(WorldQuery)]
pub struct ElementsQuery {
    pub entity: Entity,
    element: &'static Element,
}

impl<'w, 's> Deref for Elements<'w, 's> {
    type Target = Query<'w, 's, ElementsQuery, ()>;
    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl Deref for ElementsQueryItem<'_> {
    type Target = Element;
    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

#[derive(WorldQuery)]
pub struct ChildrenQuery {
    children: &'static Children,
}

// #[derive(Deref, DerefMut)]
// struct EntitiesHashSet(HashSet<Entity>);
// impl Default for EntitiesHashSet {
//     fn default() -> Self {
//         EntitiesHashSet(HashSet::new())
//     }
// }

#[derive(SystemParam)]
pub struct Elements<'w, 's> {
    pub(crate) roots: Query<'w, 's, Entity, (With<Element>, Without<Parent>)>,
    pub(crate) commands: ElementCommands<'w, 's>,
    pub(crate) elements: Query<'w, 's, ElementsQuery, ()>,
    pub(crate) children: Query<'w, 's, ChildrenQuery, ()>,
    pub(crate) id_index: Res<'w, ElementIdIndex>,
    states: Local<'s, HashMap<Entity, HashMap<Tag, bool>>>,
    classes: Local<'s, HashMap<Entity, HashSet<Tag>>>,
}

impl<'w, 's> Elements<'w, 's> {
    pub fn invalidate(&mut self, tree: Entity) {
        self.commands().add(InvalidateElementCommand(tree));
    }

    pub fn invalidate_all(&mut self) {
        self.roots
            .iter()
            .collect::<Vec<_>>()
            .iter()
            .for_each(|e| self.invalidate(*e));
    }

    pub fn entity<'e>(&'e mut self, entity: Entity) -> SelectedElements<'w, 's, 'e> {
        SelectedElements {
            elements: self,
            entities: vec![entity],
        }
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
        if selector.is_empty() {
            return SelectedElements {
                elements: self,
                entities: result,
            };
        }
        let mut branch = vec![];
        if let Some(id) = selector.get_root_id() {
            // indexed-by-id branch lookup
            if let Some(entity) = self.id_index.get(&id) {
                self.select_branch(*entity, &mut branch, &selector, &mut result);
            } else {
                warn!("Element #{id} not indexed, Elements.select() will return empty result");
            }
        } else {
            for root in self.roots.iter() {
                // branch.append(&*entuty);
                self.select_branch(root, &mut branch, &selector, &mut result);
            }
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
        let Some(old_value) = self
            .states
            .get(&entity)
            .and_then(|s| s.get(&state).copied())
            .or_else(|| {
                if let Ok(element) = self.elements.get(entity) {
                    Some(element.state.contains(&state))
                } else {
                    None
                }
            })
        else {
            return;
        };
        if value == old_value {
            return;
        }
        self.states.entry(entity).or_default().insert(state, value);
        if value {
            self.commands.add(AddStateCommand(entity, state));
        } else {
            self.commands.add(RemoveStateCommand(entity, state));
        }
        self.invalidate(entity);
    }

    pub fn add_class(&mut self, entity: Entity, class: Tag) {
        let mut element_found = true;
        let classes = self.classes.entry(entity).or_insert_with(|| {
            if let Ok(element) = self.elements.get(entity) {
                element.classes.clone()
            } else {
                element_found = false;
                HashSet::new()
            }
        });
        if !element_found || classes.contains(&class) {
            return;
        }
        classes.insert(class);
        self.commands.add(AddClassCommand(entity, class));
        self.invalidate(entity);
    }

    pub fn remove_class(&mut self, entity: Entity, class: Tag) {
        let mut element_found = true;
        let classes = self.classes.entry(entity).or_insert_with(|| {
            if let Ok(element) = self.elements.get(entity) {
                element.classes.clone()
            } else {
                element_found = false;
                HashSet::new()
            }
        });
        if !element_found || !classes.contains(&class) {
            return;
        }
        classes.remove(&class);
        self.commands.add(RemoveClassCommand(entity, class));
        self.invalidate(entity);
    }

    pub fn toggle_class(&mut self, entity: Entity, class: Tag) {
        let mut element_found = true;
        let classes = self.classes.entry(entity).or_insert_with(|| {
            if let Ok(element) = self.elements.get(entity) {
                element.classes.clone()
            } else {
                element_found = false;
                HashSet::new()
            }
        });
        if !element_found {
            return;
        }
        if classes.contains(&class) {
            classes.remove(&class);
            self.commands.add(RemoveClassCommand(entity, class));
        } else {
            classes.insert(class);
            self.commands.add(AddClassCommand(entity, class));
        }
        self.invalidate(entity);
    }

    pub fn add_child(&mut self, entity: Entity, eml: Eml) {
        self.commands.add(eml.add_to(entity));
    }

    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.commands
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
            if let Some(entity) = self.elements.commands.get_entity(entity) {
                entity.despawn_recursive();
            }
        }
    }

    /// Adds eml content to the first matched element
    pub fn add_child(&mut self, eml: Eml) -> &mut Self {
        if let Some(entity) = self.entities.first() {
            self.elements.add_child(*entity, eml);
        }
        self
    }

    pub fn add_child_with<F: FnOnce(Entity) -> Eml>(&mut self, func: F) -> &mut Self {
        if let Some(entity) = self.entities.first() {
            let child = self.elements.commands.spawn_empty().id();
            self.elements.commands.entity(*entity).add_child(child);
            self.elements.commands.add(func(child).render_to(child));
        }
        self
    }

    /// Adds eml content from `children` func to each matched element
    /// Looks like this is wrong implementation
    #[deprecated(note = "This method works weird or doesn't work at all. Do not use it.")]
    pub fn add_children<F: Fn(Entity) -> Eml>(&mut self, children: F) -> &mut Self {
        for entity in self.entities.iter().copied() {
            self.elements.add_child(entity, children(entity));
        }
        self
    }
}

#[derive(Deref, DerefMut)]
pub struct ElementCommands<'w, 's>(Commands<'w, 's>);

impl<'w, 's> ElementCommands<'w, 's> {
    pub fn new(queue: &'s mut CommandQueue, world: &'w World) -> Self {
        Self(Commands::new(queue, world))
    }
}

#[derive(Default, Deref, DerefMut)]
pub struct ElementCommandsQueue(CommandQueue);

// SAFETY: Commands only accesses internal state
unsafe impl<'w, 's> SystemParam for ElementCommands<'w, 's> {
    type State = ElementCommandsQueue;
    type Item<'world, 'state> = ElementCommands<'world, 'state>;

    fn init_state(_world: &mut World, _system_meta: &mut SystemMeta) -> Self::State {
        Default::default()
    }

    #[inline]
    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        _system_meta: &SystemMeta,
        world: UnsafeWorldCell<'world>,
        _change_tick: Tick,
    ) -> Self::Item<'world, 'state> {
        ElementCommands::new(&mut state.0, world.world())
    }

    fn apply(state: &mut Self::State, _system_meta: &SystemMeta, world: &mut World) {
        state.0.apply(world);
    }
}

pub struct InvalidateElementCommand(Entity);
impl Command for InvalidateElementCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.0) {
            entity.insert(InvalidateElement::default());
        }
    }
}

pub struct RemoveStateCommand(Entity, Tag);
impl Command for RemoveStateCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.0) {
            if let Some(mut element) = entity.get_mut::<Element>() {
                let state = self.1;
                if element.state.contains(&state) {
                    element.state.remove(&state);
                }
            }
        }
    }
}
pub struct AddStateCommand(Entity, Tag);
impl Command for AddStateCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.0) {
            if let Some(mut element) = entity.get_mut::<Element>() {
                let state = self.1;
                if !element.state.contains(&state) {
                    element.state.insert(state);
                }
            }
        }
    }
}

pub struct AddClassCommand(Entity, Tag);
impl Command for AddClassCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.0) {
            if let Some(mut element) = entity.get_mut::<Element>() {
                let class = self.1;
                if !element.classes.contains(&class) {
                    element.classes.insert(class);
                }
            }
        }
    }
}

pub struct RemoveClassCommand(Entity, Tag);
impl Command for RemoveClassCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.0) {
            if let Some(mut element) = entity.get_mut::<Element>() {
                let class = self.1;
                if element.classes.contains(&class) {
                    element.classes.remove(&class);
                }
            }
        }
    }
}

pub struct CleanupElementCommand(Entity);
impl Command for CleanupElementCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(self.0) {
            entity.remove::<(ElementBundle, TextElementBundle, ImageElementBundle)>();
        }
    }
}

#[derive(Component, Default)]
pub struct InvalidateElement;
pub fn invalidate_elements(
    invalid: Query<Entity, With<InvalidateElement>>,
    children: Query<&Children>,
    mut elements: Query<&mut Element>,
    mut invalidated: Local<HashSet<Entity>>,
    mut commands: Commands,
) {
    invalidated.clear();
    for entity in invalid.iter() {
        invalidate_children(entity, &children, &mut elements, invalidated.deref_mut());
        if let Some(mut entity) = commands.get_entity(entity) {
            entity.remove::<InvalidateElement>();
        }
    }
}

pub fn invalidate_children(
    entity: Entity,
    children: &Query<&Children>,
    elements: &mut Query<&mut Element>,
    invalidated: &mut HashSet<Entity>,
) {
    if invalidated.contains(&entity) {
        return;
    }
    invalidated.insert(entity);
    if let Ok(mut element) = elements.get_mut(entity) {
        element.invalidate();
    }
    if let Ok(chs) = children.get(entity) {
        for ch in chs.iter() {
            invalidate_children(*ch, children, elements, invalidated)
        }
    }
}
