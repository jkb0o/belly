use bevy::ecs::entity;
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::*;
use std::ops::{Deref, DerefMut};
use std::mem;

use crate::{attributes::*, ElementsBuilder};
use crate::builders::TextElementBuilder;
use crate::tags::*;

#[derive(Default)]
pub struct BuildingContext {
    stack: Vec<ContextData>
}

impl BuildingContext {
    pub (crate) fn text(&mut self) -> TextContext {
        let text = self.stack.last_mut().expect("Awaited TextContest for element");
        let text = mem::take(text);
        if let ContextData::Text(text) = text {
            text
            // text
        } else {
            panic!("Awaited TextContest for element")
        }
    }
}

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
    Text(TextContext),
}

impl ContextData {
    pub fn is_inline(&self) -> bool {
        match self {
            ContextData::Element(ctx) => ctx.inline_element,
            _ => true
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
            name, element,
            inline_element: false,
            child_elements: Default::default(),
            attributes: Default::default()
        }
    }

    pub fn inline(&mut self) {
        self.inline_element = true;
    }

    pub fn content(&mut self) -> Vec<Entity> {
        mem::take(&mut self.child_elements)
    }

    pub fn param<T:'static>(&mut self, param: &str, default: T) -> T {
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
    pub text: String
}

pub mod internal {
    use super::*;
    pub fn push_text(world: &mut World, element: Entity, text: String) {
        push_context(world, ContextData::Text(TextContext { element, text }))
    }
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
    fn into_content(self, world: &mut World) -> Vec<Entity>;
}

impl IntoContent for String {
    fn into_content(self, world: &mut World) -> Vec<Entity> {
        let text_entity = world.spawn().id();
        internal::push_text(world, text_entity, self);
        world
            .resource::<TextElementBuilder>().clone()
            .build(world);
        internal::pop_context(world);
        vec![text_entity]
    }
}

impl IntoContent for Vec<Entity> {
    fn into_content(self, _world: &mut World) -> Vec<Entity> {
        self
    }
}

impl<T: Iterator, F: Fn(T::Item) -> ElementsBuilder> IntoContent for ExpandElements<T, F> {
    fn into_content(self, world: &mut World) -> Vec<Entity> {
        let mut result = vec![];
        for builder in self {
            let entity = world.spawn().id();
            result.push(entity.clone());
            builder.with_entity(entity)(world);
        }
        result
    }
}

impl IntoContent for Vec<ElementsBuilder> {
    fn into_content(self, world: &mut World) -> Vec<Entity> {
        let mut result = vec![];
        for builder in self {
            let entity = world.spawn().id();
            result.push(entity.clone());
            builder.with_entity(entity)(world);
        }
        result
    }
}

impl IntoContent for BoxedSystem<(), ElementsBuilder> {
    fn into_content(self, world: &mut World) -> Vec<Entity> {
        vec![]
    }
}

pub struct ExpandElements<I:Iterator, F:Fn(I::Item) -> ElementsBuilder> {
    mapper: F,
    previous: I
}

impl<I, F> Iterator for ExpandElements<I, F>
where
    I: Iterator,
    F:Fn(I::Item) -> ElementsBuilder
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
    fn elements<F:Fn(Self::Item) -> ElementsBuilder>(self, mapper:F) -> ExpandElements<Self, F> 
    where
        Self: Sized
    {
        ExpandElements { mapper, previous: self }

    }
}

impl<I: Iterator> ExpandElementsExt for I {}

 
// impl<I: Iterator> ExpandExt for I {}

fn test() {
    // ["1", "2"].iter().elements(|e| bsx!{ <el>{e}</el>})
}