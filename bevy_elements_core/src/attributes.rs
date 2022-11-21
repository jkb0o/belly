use std::{
    any::{Any, TypeId},
    fmt::Debug,
    mem,
};

use super::ElementsBuilder;
use crate::{property::*, bind::{BindFrom, BindValue, BindFromUntyped}, BuildingContext};
use crate::tags;
use tagstr::*;
use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    utils::{hashbrown::hash_map::Drain, HashMap, HashSet},
};

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
    BindFrom(BindFromUntyped)
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
            AttributeValue::BindFrom(_) => write!(f, "AttributeValue::BindFrom"),
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

// pub trait AddAttribute<T:'static> {
//     type Params;
//     fn add<Params>(&mut self, name: &str, attr: T);
// } 

// struct Attrs;
// impl<F: IntoSystem<(), (), Self::Params> + 'static> AddAttribute<F> for Attrs {
//     fn add(&mut self, name: &str, attr: F) {
        
//     }
//     // type Params = Params;

// }

pub trait IntoAttr<Params> {
    fn into_attr(value: Self) -> AttributeValue;
}

pub trait IntoCommands {

}

impl<Params, S: IntoSystem<(), (), Params>> IntoAttr<Params> for S {
    fn into_attr(value: Self) -> AttributeValue {
        let s = IntoSystem::into_system(value);
        AttributeValue::Empty
    }
}

pub struct WithoutParams;

impl IntoAttr<WithoutParams> for String {
    fn into_attr(value: Self) -> AttributeValue {
        AttributeValue::Empty
    }
}

impl IntoAttr<WithoutParams> for i32 {
    fn into_attr(value: Self) -> AttributeValue {
        AttributeValue::Empty
    }
}

pub struct AttributeCommands(Box<dyn FnOnce(&mut EntityCommands)>);
impl AttributeCommands {
    pub fn new<F: FnOnce(&mut EntityCommands) + 'static>(commands: F) -> AttributeCommands {
        AttributeCommands(Box::new(commands))
    }
}

impl<F: FnOnce(&mut EntityCommands) + 'static> From<F> for AttributeCommands {
    fn from(f: F) -> Self {
        AttributeCommands::new(f)
    }
}

pub struct NoFunc;
impl IntoAttr<WithoutParams> for AttributeCommands {
    fn into_attr(value: Self) -> AttributeValue {
        AttributeValue::Empty
    }
}

fn x() { }
fn t2<Params, S: IntoSystem<(), (), Params>>(s: S) {
    let x = IntoSystem::into_system(s);
}
fn test() {
    IntoAttr::into_attr(|mut commands: Commands| {});
}

// fn test<T: 'static>(value: T) {
//     {
//         let a = (&value as &dyn Any).downcast_ref::<IntoSystem<(), (), i32>>();
//     }
// }

// impl<S: IntoSystem<(), (), SystemParam>> IntoAttr for S {
//     fn into_attr(value: Self) -> AttributeValue {
//         let s: BoxedSystem<(), ()> = Box::new(IntoSystem::into_system(value));
//     }
// }

impl AttributeValue {
    // pub fn new<T:Any + 'static>(value: T) {
    //     let boxed: Box<dyn Any> = Box::new(value);
    //     let boxed = boxed.downcast::<BoxedSystem<(), ()>>().unwrap();
    //     let value = Some(*boxed);
    // }
    pub fn is<T: 'static>(&self) -> bool {
        match self {
            AttributeValue::Empty => false,
            AttributeValue::Int(_) => TypeId::of::<T>() == TypeId::of::<i32>(),
            AttributeValue::String(_) => TypeId::of::<T>() == TypeId::of::<String>(),
            AttributeValue::Entity(_) => TypeId::of::<T>() == TypeId::of::<Entity>(),
            AttributeValue::Commands(_) => TypeId::of::<T>() == TypeId::of::<ApplyCommands>(),
            AttributeValue::Elements(_) => TypeId::of::<T>() == TypeId::of::<ElementsBuilder>(),
            AttributeValue::Attributes(_) => TypeId::of::<T>() == TypeId::of::<Attributes>(),
            AttributeValue::BindFrom(_) => TypeId::of::<T>() == TypeId::of::<BindFromUntyped>(),
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
            AttributeValue::BindFrom(v) => try_cast::<T, BindFromUntyped>(v),

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
            AttributeValue::BindFrom(v) => try_cast_mut::<T, BindFromUntyped>(v),
        }
    }

    pub fn take<T: 'static>(self) -> Option<T> {
        match self {
            AttributeValue::Empty => None,
            AttributeValue::Int(v) => try_take::<T, i32>(v),
            AttributeValue::String(v) => try_take::<T, String>(v),
            AttributeValue::Entity(v) => try_take::<T, Entity>(v),
            AttributeValue::Commands(v) => try_take::<T, ApplyCommands>(v),
            AttributeValue::Elements(v) => try_take::<T, ElementsBuilder>(v),
            AttributeValue::Attributes(v) => try_take::<T, Attributes>(v),
            AttributeValue::BindFrom(v) => try_take::<T, BindFromUntyped>(v),
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
    pub fn from_commands(name: &str, commands: ApplyCommands) -> Attribute {
        let value = AttributeValue::Commands(commands);
        Attribute { name: name.as_tag(), value, target: AttributeTarget::Attribute }
    }
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
                value,
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

    pub fn take_varint(&mut self) -> AttributeValue {
        mem::take(&mut self.value)
    }
}

// fn test_system
#[derive(Default, Debug)]
pub struct Attributes(HashMap<Tag, Attribute>);

impl Attributes {
    pub fn add(&mut self, mut attr: Attribute) {
        if attr.name == tags::params() {
            if let Some(mut attrs) = attr.take::<Attributes>() {
                let this = mem::take(self);
                attrs.merge(this);
                *self = attrs;
                return;
            } else {
                panic!("Params attribute should an Attributes type.")
            }
        }
        if attr.name == tags::class() {
            if let Some(existed) = self.get_mut::<String>(attr.name) {
                if let Some(classes) = attr.take::<String>() {
                    existed.push_str(" ");
                    existed.push_str(classes.as_str());
                    return;
                }
            }
        }
        match attr.target {
            AttributeTarget::Attribute => self.0.insert(attr.name, attr),
            AttributeTarget::Class => match self.0.get_mut(&tags::class()) {
                Some(class) => {
                    let classes = class
                        .value
                        .get_mut::<String>()
                        .expect("Class attribute should be a String type");
                    classes.push_str(" ");
                    classes.push_str(attr.name.into());
                    println!("extending existed class attribute {:?}", attr.name);
                    None
                }
                None => {
                    println!("adding new class attribute {:?}", attr.name);
                    attr = Attribute::new(
                        tags::class().into(),
                        AttributeValue::String(attr.name.into()),
                    );
                    self.0.insert(tags::class(), attr)
                }
            },
            AttributeTarget::Style => match self.0.get_mut(&tags::styles()) {
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
                        Attribute::new(tags::styles().into(), AttributeValue::Attributes(styles));
                    self.0.insert(tags::styles(), attr)
                }
            },
        };
    }

    pub fn drain(&mut self) -> Drain<Tag, Attribute> {
        self.0.drain()
    }

    pub fn merge(&mut self, mut other: Self) {
        if let Some(other_classes) = other.0.remove(&tags::class()) {
            if let Some(self_classes) = self.0.get_mut(&tags::class()) {
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
                self.0.insert(tags::class(), other_classes);
            }
        }
        if let Some(mut other_styles) = other.0.remove(&tags::styles()) {
            if let Some(self_styles) = self.0.get_mut(&tags::styles()) {
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
                self.0.insert(tags::styles(), other_styles);
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
        self.drop::<ApplyCommands>(name)
    }

    pub fn styles(&mut self) -> HashMap<Tag, PropertyValues> {
        if let Some(mut styles) = self.drop::<Attributes>(tags::styles()) {
            let mut result: HashMap<Tag, PropertyValues> = Default::default();
            for (tag, attr) in styles.drain() {
                if let Some(value) = attr.value.get::<String>() {
                    match value.as_str().try_into() {
                        Ok(parsed) => result.insert(tag, parsed),
                        Err(err) => {
                            panic!("Unable to parse style {} for key {}: {}", value, tag, err)
                        }
                    };
                } else {
                    panic!(
                        "For now only String supported as style values, {} got {:?}",
                        tag, attr
                    );
                }
            }
            result
        } else {
            Default::default()
        }
    }
    pub fn classes(&mut self) -> HashSet<Tag> {
        self.drop::<String>(tags::class())
            .unwrap_or("".to_string())
            .split(" ")
            .filter(|s| !s.is_empty())
            .map(|s| s.as_tag())
            .collect()
    }
    pub fn id(&mut self) -> Option<Tag> {
        self.drop(tags::id())
    }
    pub fn get<T: 'static>(&self, key: Tag) -> Option<&T> {
        self.0.get(&key).and_then(|v| v.value.get::<T>())
    }
    pub fn get_variant(&self, key: Tag) -> Option<&AttributeValue> {
        self.0.get(&key).map(|v| &v.value)
    }
    pub fn get_mut<T: 'static>(&mut self, key: Tag) -> Option<&mut T> {
        self.0.get_mut(&key).and_then(|v| v.value.get_mut::<T>())
    }
    pub fn drop<T: 'static>(&mut self, key: Tag) -> Option<T> {
        self.0.remove(&key).and_then(|mut a| a.take())
    }
    pub fn drop_variant(&mut self, key: Tag) -> Option<AttributeValue> {
        self.0.remove(&key).map(|mut a| a.take_varint())
    }
    pub fn drop_or_default<T: 'static>(&mut self, key: Tag, default: T) -> T {
        if let Some(value) = self.drop(key) {
            value
        } else {
            default
        }
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

impl<W:Component, T:BindValue> From<BindFrom<W, T>> for AttributeValue {
    fn from(bind: BindFrom<W, T>) -> Self {
        AttributeValue::BindFrom(bind.to_untyped())   
    }
}


#[macro_export]
macro_rules! bindattr {
    ($ctx:ident, $cmd:ident, $key:ident:$typ:ident => $($target:tt)*) => {
        {
            let elem = $ctx.element.clone();
            let key = stringify!($key).as_tag();
            let attr = $ctx.attributes.drop_variant(key);
            let mut value = Default::default();
            match attr {
                Some(AttributeValue::BindFrom(b)) => $cmd.add(b.to($crate::bind!(=> elem, $($target)*))),
                Some(AttributeValue::$typ(v)) => value = Some(v),
                Some(attr) => error!("Unsupported value for '{}' attribute: {:?}", key, attr),
                _ => ()
            };
            value
        }
    };
}

pub(crate) use bindattr;


use bevy::prelude::*;
fn ta(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands,
) {
    let x = bindattr!(ctx, commands, value:String => Text.sections[0].value);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_classes() {
        let mut attrs = Attributes::default();
        attrs.add(Attribute::new("class", "some-class".into()));
        assert_eq!(
            attrs.classes(),
            ["some-class".as_tag()].iter().cloned().collect()
        );
    }

    #[test]
    fn test_c_not_overrides_class() {
        let mut attrs = Attributes::default();
        attrs.add(Attribute::new("class", "class1 class2".into()));
        attrs.add(Attribute::new("c:some-other-class", AttributeValue::Empty));
        assert_eq!(
            attrs.classes(),
            ["class1", "class2", "some-other-class"]
                .iter()
                .map(|s| s.as_tag())
                .collect()
        );
    }

    #[test]
    fn test_class_not_overrides_c() {
        let mut attrs = Attributes::default();
        attrs.add(Attribute::new("c:some-other-class", AttributeValue::Empty));
        attrs.add(Attribute::new("class", "class1 class2".into()));
        assert_eq!(
            attrs.classes(),
            ["class1", "class2", "some-other-class"]
                .iter()
                .map(|s| s.as_tag())
                .collect()
        );
    }

    #[test]
    fn test_basic_styles() {
        let mut attrs = Attributes::default();
        attrs.add(Attribute::new("s:color", "black".into()));
        let styles = attrs.drop::<Attributes>("styles".as_tag());
        assert!(styles.is_some());
        let styles = styles.unwrap();
        assert_eq!(styles.get::<String>("color".as_tag()), Some(&"black".to_string()));
    }
}
