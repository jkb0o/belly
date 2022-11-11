use bevy::prelude::*;
use std::ops::{Deref, DerefMut};
use std::mem;

use crate::attributes::*;
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