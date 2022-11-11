use std::{
    any::{Any, TypeId},
    fmt::Debug,
    mem,
};

use super::ElementsBuilder;
use crate::tags::*;
use crate::property::*;
use bevy::{ecs::system::EntityCommands, prelude::*, utils::{HashMap, HashSet, hashbrown::hash_map::{Iter, Values, Drain}},};

pub type ApplyCommands = Box<dyn FnOnce(&mut EntityCommands)>;

#[derive(Default)]
pub enum AttributeValue {
    #[default]
    Empty,
    Int(i32),
    String(String),
    Entity(Entity),
    Commands(ApplyCommands),
    Elements(ElementsBuilder),
    Attributes(Attributes),
}

impl Debug for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::Empty => write!(f, "AttributeValue::Empty"),
            AttributeValue::Int(v) => write!(f, "AttributeValue::Int({:?})", v),
            AttributeValue::String(v) => write!(f, "AttributeValue::String({:?})", v),
            AttributeValue::Entity(v) => write!(f, "AttributeValue::Entity({:?})", v),
            AttributeValue::Attributes(v) => write!(f, "AttributeValue::Attributes({:?})", v),
            AttributeValue::Commands(_) => write!(f, "AttributeValue::Commands"),
            AttributeValue::Elements(_) => write!(f, "AttributeValue::Elements"),
        }
    }
}

fn try_cast<T: 'static, F: 'static>(v: &dyn Any) -> Option<&T> {
    if TypeId::of::<T>() == TypeId::of::<F>() {
        v.downcast_ref::<T>()
    } else {
        None
    }
}
fn try_cast_mut<T: 'static, F: 'static>(v: &mut dyn Any) -> Option<&mut T> {
    if TypeId::of::<T>() == TypeId::of::<F>() {
        v.downcast_mut::<T>()
    } else {
        None
    }
}

fn try_take<T: 'static, F: 'static>(v: F) -> Option<T> {
    if TypeId::of::<T>() == TypeId::of::<F>() {
        let boxed: Box<dyn Any> = Box::new(v);
        let boxed = boxed.downcast::<T>().unwrap();
        Some(*boxed)
    } else {
        None
    }
}

impl AttributeValue {
    pub fn is<T: 'static>(&self) -> bool {
        match self {
            AttributeValue::Empty => false,
            AttributeValue::Int(_) => TypeId::of::<T>() == TypeId::of::<i32>(),
            AttributeValue::String(_) => TypeId::of::<T>() == TypeId::of::<String>(),
            AttributeValue::Entity(_) => TypeId::of::<T>() == TypeId::of::<Entity>(),
            AttributeValue::Commands(_) => TypeId::of::<T>() == TypeId::of::<ApplyCommands>(),
            AttributeValue::Elements(_) => TypeId::of::<T>() == TypeId::of::<ElementsBuilder>(),
            AttributeValue::Attributes(_) => TypeId::of::<T>() == TypeId::of::<Attributes>(),
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        match self {
            AttributeValue::Empty => None,
            AttributeValue::Int(v) => try_cast::<T, i32>(v),
            AttributeValue::String(v) => try_cast::<T, String>(v),
            AttributeValue::Entity(v) => try_cast::<T, Entity>(v),
            AttributeValue::Commands(v) => try_cast::<T, ApplyCommands>(v),
            AttributeValue::Elements(v) => try_cast::<T, ElementsBuilder>(v),
            AttributeValue::Attributes(v) => try_cast::<T, Attributes>(v),
        }
    }
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        match self {
            AttributeValue::Empty => None,
            AttributeValue::Int(v) => try_cast_mut::<T, i32>(v),
            AttributeValue::String(v) => try_cast_mut::<T, String>(v),
            AttributeValue::Entity(v) => try_cast_mut::<T, Entity>(v),
            AttributeValue::Commands(v) => try_cast_mut::<T, ApplyCommands>(v),
            AttributeValue::Elements(v) => try_cast_mut::<T, ElementsBuilder>(v),
            AttributeValue::Attributes(v) => try_cast_mut::<T, Attributes>(v),
        }
    }

    fn take<T: 'static>(self) -> Option<T> {
        match self {
            AttributeValue::Empty => None,
            AttributeValue::Int(v) => try_take::<T, i32>(v),
            AttributeValue::String(v) => try_take::<T, String>(v),
            AttributeValue::Entity(v) => try_take::<T, Entity>(v),
            AttributeValue::Commands(v) => try_take::<T, ApplyCommands>(v),
            AttributeValue::Elements(v) => try_take::<T, ElementsBuilder>(v),
            AttributeValue::Attributes(v) => try_take::<T, Attributes>(v),
        }
    }

    pub fn merge(&mut self, other: Self) {
        let this = mem::take(self);
        *self = match (this, other) {
            (AttributeValue::Commands(a), AttributeValue::Commands(b)) => {
                AttributeValue::Commands(Box::new(move |commands: &mut EntityCommands| {
                    a(commands);
                    b(commands);
                }))
            }
            (AttributeValue::Attributes(mut a), AttributeValue::Attributes(b)) => {
                a.merge(b);
                AttributeValue::Attributes(a)
            }
            (_, other) => other,
        }
    }
}

unsafe impl Sync for AttributeValue {}
unsafe impl Send for AttributeValue {}

#[derive(Debug)]
pub enum AttributeTarget {
    Attribute,
    Style,
    Class,
}

#[derive(Debug)]
pub struct Attribute {
    name: Tag,
    value: AttributeValue,
    target: AttributeTarget,
}

impl Attribute {
    pub fn new(name: &str, value: AttributeValue) -> Attribute {
        if name.starts_with("c:") {
            Attribute {
                name: name.strip_prefix("c:").unwrap().as_tag(),
                value: AttributeValue::Empty,
                target: AttributeTarget::Class,
            }
        } else if name.starts_with("s:") {
            Attribute {
                name: name.strip_prefix("s:").unwrap().as_tag(),
                value: AttributeValue::Empty,
                target: AttributeTarget::Style,
            }
        } else {
            Attribute {
                value,
                name: name.as_tag(),
                target: AttributeTarget::Attribute,
            }
        }
    }

    pub fn take<T: 'static>(&mut self) -> Option<T> {
        mem::take(&mut self.value).take()
    }
}

// fn test_system
#[derive(Default, Debug)]
pub struct Attributes(HashMap<Tag, Attribute>);

impl Attributes {
    pub fn add(&mut self, mut attr: Attribute) {
        if attr.name == Tag::params() {
            if let Some(mut attrs) = attr.take::<Attributes>() {
                let this = mem::take(self);
                attrs.merge(this);
                *self = attrs;
                return;
            } else {
                panic!("Params attribute should an Attributes type.")
            }
        }
        match attr.target {
            AttributeTarget::Attribute => self.0.insert(attr.name, attr),
            AttributeTarget::Class => match self.0.get_mut(&Tag::class()) {
                Some(class) => {
                    let classes = class
                        .value
                        .get_mut::<String>()
                        .expect("Class attribute should be a String type");
                    classes.push_str(" ");
                    classes.push_str(attr.name.into());
                    None
                }
                None => {
                    // println!("adding class attribute {:?}", attr.name);
                    attr = Attribute::new(
                        Tag::class().into(),
                        AttributeValue::String(attr.name.into()),
                    );
                    self.0.insert(Tag::class(), attr)
                }
            },
            AttributeTarget::Style => match self.0.get_mut(&Tag::styles()) {
                Some(styles) => {
                    let styles = styles
                        .value
                        .get_mut::<Attributes>()
                        .expect("Styles attribute should be Attributes type");
                    attr.target = AttributeTarget::Attribute;
                    styles.add(attr);
                    None
                }
                None => {
                    let mut styles = Attributes::default();
                    attr.target = AttributeTarget::Attribute;
                    styles.add(attr);
                    let attr =
                        Attribute::new(Tag::styles().into(), AttributeValue::Attributes(styles));
                    self.0.insert(Tag::styles(), attr)
                }
            },
        };
    }

    pub fn drain(&mut self) -> Drain<Tag, Attribute> {
        self.0.drain()
    }

    pub fn merge(&mut self, mut other: Self) {
        if let Some(other_classes) = other.0.remove(&Tag::class()) {
            if let Some(self_classes) = self.0.get_mut(&Tag::class()) {
                let self_class_string = self_classes
                    .value
                    .get_mut::<String>()
                    .expect("Class attribute should be a String type");
                let other_class_string = other_classes
                    .value
                    .get::<String>()
                    .expect("Class attribute should be a String type");
                self_class_string.push_str(" ");
                self_class_string.push_str(other_class_string.as_str());
            } else {
                self.0.insert(Tag::class(), other_classes);
            }
        }
        if let Some(mut other_styles) = other.0.remove(&Tag::styles()) {
            if let Some(self_styles) = self.0.get_mut(&Tag::styles()) {
                let self_styles_value = self_styles
                    .value
                    .get_mut::<Attributes>()
                    .expect("Styles attribute should be an Attributes type");
                let other_styles_value = other_styles
                    .value
                    .get_mut::<Attributes>()
                    .expect("Styles attribute should be an Attributes type");
                for (_, attr) in other_styles_value.0.drain() {
                    self_styles_value.add(attr);
                }
            } else {
                self.0.insert(Tag::styles(), other_styles);
            }
        }
        for (name, attr) in other.0.drain() {
            if let Some(self_attr) = self.0.get_mut(&name) {
                self_attr.value.merge(attr.value);
            } else {
                self.add(attr);
            }
        }
    }

    pub fn commands(&mut self, name: Tag) -> Option<ApplyCommands> {
        self.remove::<ApplyCommands>(name)
    }

    pub fn styles(&mut self) -> HashMap<Tag, PropertyValues> {
        if let Some(mut styles) = self.remove::<Attributes>(Tag::styles()) {
            let mut result: HashMap<Tag, PropertyValues> = Default::default();
            for (tag, attr) in styles.drain() {
                if let Some(value) = attr.value.get::<String>() {
                    match value.as_str().try_into() {
                        Ok(parsed) => result.insert(tag, parsed),
                        Err(err) => panic!("Unable to parse style {} for key {}: {}", value, tag, err)
                    };
                } else {
                    panic!("For now only String supported as style values, {} got {:?}", tag, attr);
                }
            }
            result
        } else {
            Default::default()
        }
    }
    pub fn classes(&mut self) -> HashSet<Tag> {
        self.remove::<String>(Tag::class())
            .unwrap_or("".to_string())
            .split(" ")
            .filter(|s| !s.is_empty())
            .map(|s| s.as_tag())
            .collect()
    }
    pub fn id(&mut self) -> Option<String> {
        self.remove(Tag::id())
    }
    pub fn get<T: 'static>(&self, key: Tag) -> Option<&T> {
        self.0.get(&key).and_then(|v| v.value.get::<T>())
    }
    fn remove<T: 'static>(&mut self, key: Tag) -> Option<T> {
        self.0.remove(&key).and_then(|mut a| a.take())
    }
    pub fn apply_commands(&mut self, for_attribute: Tag, commands: &mut EntityCommands) {
        if let Some(attribute_commands) = self.commands(for_attribute) {
            attribute_commands(commands)
        }
    }
}

pub struct InsertComponent<T: Component>(T);

pub fn component<T: Component>(component: T) -> InsertComponent<T> {
    InsertComponent(component)
}

impl From<i32> for AttributeValue {
    fn from(v: i32) -> Self {
        AttributeValue::Int(v)
    }
}

impl From<String> for AttributeValue {
    fn from(v: String) -> Self {
        AttributeValue::String(v)
    }
}

impl From<&str> for AttributeValue {
    fn from(v: &str) -> Self {
        AttributeValue::String(v.to_string())
    }
}

impl From<Entity> for AttributeValue {
    fn from(v: Entity) -> Self {
        AttributeValue::Entity(v)
    }
}

impl From<ApplyCommands> for AttributeValue {
    fn from(commands: ApplyCommands) -> Self {
        AttributeValue::Commands(commands)
    }
}

impl From<Attributes> for AttributeValue {
    fn from(v: Attributes) -> Self {
        AttributeValue::Attributes(v)
    }
}
