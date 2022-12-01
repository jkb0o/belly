use bevy::ecs::system::Command;
use bevy::prelude::*;
use std::any::TypeId;
use std::mem;
use std::ops::{Deref, DerefMut};

use crate::tags::*;
use crate::{attributes::*, ElementsBuilder};
use crate::{bind::*, Element};

#[derive(Default, Resource)]
pub struct BuildingContext {
    stack: Vec<ContextData>,
}

impl BuildingContext {}

impl Deref for BuildingContext {
    type Target = ElementContext;

    fn deref(&self) -> &Self::Target {
        if let Some(ContextData::Element(ctx)) = self.stack.iter().last() {
            ctx
        } else {
            panic!("Invalid ElementsContext state")
        }
    }
}

impl DerefMut for BuildingContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(ContextData::Element(ctx)) = self.stack.iter_mut().last() {
            ctx
        } else {
            panic!("Invalid ElementsContext state")
        }
    }
}

#[derive(Debug, Default)]
pub enum ContextData {
    #[default]
    Empty,
    Element(ElementContext),
}

impl ContextData {
    pub fn is_inline(&self) -> bool {
        match self {
            ContextData::Element(ctx) => ctx.inline_element,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct ElementContext {
    pub name: Tag,
    pub element: Entity,
    inline_element: bool,
    child_elements: Vec<Entity>,
    pub attributes: Attributes,
}

impl ElementContext {
    pub fn new(name: Tag, element: Entity) -> ElementContext {
        ElementContext {
            name,
            element,
            inline_element: false,
            child_elements: Default::default(),
            attributes: Default::default(),
        }
    }

    pub fn inline(&mut self) {
        self.inline_element = true;
    }

    pub fn content(&mut self) -> Vec<Entity> {
        mem::take(&mut self.child_elements)
    }

    pub fn param<T: 'static>(&mut self, param: &str, default: T) -> T {
        self.attributes.drop_or_default(param.as_tag(), default)
    }

    pub fn params(&mut self) -> Attributes {
        mem::take(&mut self.attributes)
    }

    pub fn add_child(&mut self, child: Entity) {
        self.child_elements.push(child);
    }
}

#[derive(Debug)]
pub struct TextContext {
    pub element: Entity,
    pub text: String,
}

pub mod internal {
    use super::*;
    pub fn push_element(world: &mut World, element: ElementContext) {
        push_context(world, ContextData::Element(element))
    }
    pub fn push_context(world: &mut World, data: ContextData) {
        let mut ctx = world.get_resource_or_insert_with(BuildingContext::default);
        ctx.stack.push(data);
    }

    pub fn pop_context(world: &mut World) -> Option<ContextData> {
        let mut ctx = world.get_resource_or_insert_with(BuildingContext::default);
        let data = ctx.stack.pop();
        if data.is_none() {
            world.remove_resource::<BuildingContext>();
        }
        data
    }
}

pub trait IntoContent {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity>;
}

impl IntoContent for String {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity> {
        let mut entity = world.entity_mut(parent);
        if let Some(mut text) = entity.get_mut::<Text>() {
            text.sections[0].value = self;
        } else {
            let text = Text::from_section(self, Default::default());
            entity
                .insert(Element::inline())
                .insert(TextBundle { text, ..default() });
        }
        vec![parent]
    }
}

#[derive(Component)]
struct BindContent<T: BindValue + IntoContent + std::fmt::Debug> {
    value: T,
}
impl<C: Component, T: BindValue + IntoContent + std::fmt::Debug> IntoContent for BindFrom<C, T> {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity> {
        world
            .entity_mut(parent)
            .insert(NodeBundle::default())
            .insert(BindContent {
                value: T::default(),
            });
        let target = BindTo::new(
            parent,
            |t: &BindContent<T>, v| &t.value == v,
            |t: &mut BindContent<T>, v| t.value.clone_from(v),
        );
        let bind = Bind::new(self, target);
        bind.write(world);
        let systems_ref = world.get_resource_or_insert_with(BindingSystems::default);
        let mut systems = systems_ref.0.write().unwrap();
        systems.add_custom_system(TypeId::of::<BindContent<T>>(), bind_component_system::<T>);

        vec![parent]
    }
}

fn bind_component_system<T: BindValue + IntoContent + std::fmt::Debug>(
    mut commands: Commands,
    binds: Query<(Entity, &BindContent<T>), Changed<BindContent<T>>>,
) {
    // info!("bindsystem for {}", type_name::<T>());
    for (entity, bind) in binds.iter() {
        let content = bind.value.clone();
        info!("bind value changed for {:?}", entity);
        commands.add(move |world: &mut World| {
            info!("setting value: bind value changed to {:?}", content);
            content.into_content(entity, world);
        })
    }
}

impl IntoContent for Vec<Entity> {
    fn into_content(self, _parent: Entity, _world: &mut World) -> Vec<Entity> {
        self
    }
}

impl<T: Iterator, F: Fn(T::Item) -> ElementsBuilder> IntoContent for ExpandElements<T, F> {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        let mut result = vec![];
        for builder in self {
            let entity = world.spawn_empty().id();
            result.push(entity.clone());
            builder.with_entity(entity)(world);
        }
        result
    }
}

impl IntoContent for Vec<ElementsBuilder> {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        let mut result = vec![];
        for builder in self {
            let entity = world.spawn_empty().id();
            result.push(entity.clone());
            builder.with_entity(entity)(world);
        }
        result
    }
}

impl IntoContent for ElementsBuilder {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity> {
        self.with_entity(parent)(world);
        vec![parent]
    }
}

pub struct ExpandElements<I: Iterator, F: Fn(I::Item) -> ElementsBuilder> {
    mapper: F,
    previous: I,
}

impl<I, F> Iterator for ExpandElements<I, F>
where
    I: Iterator,
    F: Fn(I::Item) -> ElementsBuilder,
{
    type Item = ElementsBuilder;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(x) = self.previous.next() {
            return Some((self.mapper)(x));
        }
        None
    }
}

pub trait ExpandElementsExt: Iterator {
    fn elements<F: Fn(Self::Item) -> ElementsBuilder>(self, mapper: F) -> ExpandElements<Self, F>
    where
        Self: Sized,
    {
        ExpandElements {
            mapper,
            previous: self,
        }
    }
}

impl<I: Iterator> ExpandElementsExt for I {}
