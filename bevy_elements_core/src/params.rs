use std::{
    any::{type_name, Any, TypeId},
    fmt::Debug,
    mem,
};

use crate::eml::build::ElementsBuilder;
use crate::tags;
use crate::{
    property::*,
    relations::{BindFrom, BindFromUntyped, BindTo, BindToUntyped, BindValue},
};
use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    utils::{hashbrown::hash_map::Drain, HashMap, HashSet},
};
use tagstr::*;

pub type ApplyCommands = Box<dyn FnOnce(&mut EntityCommands)>;

#[derive(Default)]
pub enum Variant {
    #[default]
    Empty,
    Int(i32),
    String(String),
    Entity(Entity),
    Commands(ApplyCommands),
    Elements(ElementsBuilder),
    Params(Params),
    BindFrom(BindFromUntyped),
    BindTo(BindToUntyped),
    Any(Box<dyn Any>),
}

impl Debug for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::Empty => write!(f, "Variant::Empty"),
            Variant::Int(v) => write!(f, "Variant::Int({:?})", v),
            Variant::String(v) => write!(f, "Variant::String({:?})", v),
            Variant::Entity(v) => write!(f, "Variant::Entity({:?})", v),
            Variant::Params(v) => write!(f, "Variant::Params({:?})", v),
            Variant::Commands(_) => write!(f, "Variant::Commands"),
            Variant::Elements(_) => write!(f, "Variant::Elements"),
            Variant::BindFrom(_) => write!(f, "Variant::BindFrom"),
            Variant::BindTo(_) => write!(f, "Variant::BindTo"),
            Variant::Any(_) => write!(f, "Variant::Any"),
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

impl Variant {
    // pub fn new<T:Any + 'static>(value: T) {
    //     let boxed: Box<dyn Any> = Box::new(value);
    //     let boxed = boxed.downcast::<BoxedSystem<(), ()>>().unwrap();
    //     let value = Some(*boxed);
    // }
    pub fn is<T: 'static>(&self) -> bool {
        match self {
            Variant::Empty => false,
            Variant::Int(_) => TypeId::of::<T>() == TypeId::of::<i32>(),
            Variant::String(_) => TypeId::of::<T>() == TypeId::of::<String>(),
            Variant::Entity(_) => TypeId::of::<T>() == TypeId::of::<Entity>(),
            Variant::Commands(_) => TypeId::of::<T>() == TypeId::of::<ApplyCommands>(),
            Variant::Elements(_) => TypeId::of::<T>() == TypeId::of::<ElementsBuilder>(),
            Variant::Params(_) => TypeId::of::<T>() == TypeId::of::<Params>(),
            Variant::BindFrom(_) => TypeId::of::<T>() == TypeId::of::<BindFromUntyped>(),
            Variant::BindTo(_) => TypeId::of::<T>() == TypeId::of::<BindToUntyped>(),
            Variant::Any(v) => v.is::<T>(),
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        match self {
            Variant::Empty => None,
            Variant::Int(v) => try_cast::<T, i32>(v),
            Variant::String(v) => try_cast::<T, String>(v),
            Variant::Entity(v) => try_cast::<T, Entity>(v),
            Variant::Commands(v) => try_cast::<T, ApplyCommands>(v),
            Variant::Elements(v) => try_cast::<T, ElementsBuilder>(v),
            Variant::Params(v) => try_cast::<T, Params>(v),
            Variant::BindFrom(v) => try_cast::<T, BindFromUntyped>(v),
            Variant::BindTo(v) => try_cast::<T, BindToUntyped>(v),
            Variant::Any(v) => v.downcast_ref::<T>(),
        }
    }
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        match self {
            Variant::Empty => None,
            Variant::Int(v) => try_cast_mut::<T, i32>(v),
            Variant::String(v) => try_cast_mut::<T, String>(v),
            Variant::Entity(v) => try_cast_mut::<T, Entity>(v),
            Variant::Commands(v) => try_cast_mut::<T, ApplyCommands>(v),
            Variant::Elements(v) => try_cast_mut::<T, ElementsBuilder>(v),
            Variant::Params(v) => try_cast_mut::<T, Params>(v),
            Variant::BindFrom(v) => try_cast_mut::<T, BindFromUntyped>(v),
            Variant::BindTo(v) => try_cast_mut::<T, BindToUntyped>(v),
            Variant::Any(v) => v.downcast_mut::<T>(),
        }
    }

    pub fn take<T: 'static>(self) -> Option<T> {
        match self {
            Variant::Empty => None,
            Variant::Int(v) => try_take::<T, i32>(v),
            Variant::String(v) => try_take::<T, String>(v),
            Variant::Entity(v) => try_take::<T, Entity>(v),
            Variant::Commands(v) => try_take::<T, ApplyCommands>(v),
            Variant::Elements(v) => try_take::<T, ElementsBuilder>(v),
            Variant::Params(v) => try_take::<T, Params>(v),
            Variant::BindFrom(v) => try_take::<T, BindFromUntyped>(v),
            Variant::BindTo(v) => try_take::<T, BindToUntyped>(v),
            Variant::Any(v) => match v.downcast::<T>() {
                Ok(v) => Some(*v),
                Err(v) => {
                    error!("Can't cast {:?} to {}", v, type_name::<T>());
                    None
                }
            },
        }
    }

    pub fn merge(&mut self, other: Self) {
        let this = mem::take(self);
        *self = match (this, other) {
            (Variant::Commands(a), Variant::Commands(b)) => {
                Variant::Commands(Box::new(move |commands: &mut EntityCommands| {
                    a(commands);
                    b(commands);
                }))
            }
            (Variant::Params(mut a), Variant::Params(b)) => {
                a.merge(b);
                Variant::Params(a)
            }
            (_, other) => other,
        }
    }
}

unsafe impl Sync for Variant {}
unsafe impl Send for Variant {}

#[derive(Debug)]
pub enum ParamTarget {
    Param,
    Style,
    Class,
}

#[derive(Debug)]
pub struct Param {
    name: Tag,
    value: Variant,
    target: ParamTarget,
}

impl Param {
    pub fn from_commands(name: &str, commands: ApplyCommands) -> Param {
        let value = Variant::Commands(commands);
        Param {
            name: name.as_tag(),
            value,
            target: ParamTarget::Param,
        }
    }
    pub fn new(name: &str, value: Variant) -> Param {
        if name.starts_with("c:") {
            Param {
                name: name.strip_prefix("c:").unwrap().as_tag(),
                value: Variant::Empty,
                target: ParamTarget::Class,
            }
        } else if name.starts_with("s:") {
            Param {
                name: name.strip_prefix("s:").unwrap().as_tag(),
                value,
                target: ParamTarget::Style,
            }
        } else {
            Param {
                value,
                name: name.as_tag(),
                target: ParamTarget::Param,
            }
        }
    }

    pub fn take<T: 'static>(&mut self) -> Option<T> {
        mem::take(&mut self.value).take()
    }

    pub fn take_varint(&mut self) -> Variant {
        mem::take(&mut self.value)
    }
}

// fn test_system
#[derive(Default, Debug)]
pub struct Params(HashMap<Tag, Param>);

impl Params {
    pub fn add(&mut self, mut attr: Param) {
        if attr.name == tags::params() {
            if let Some(mut attrs) = attr.take::<Params>() {
                let this = mem::take(self);
                attrs.merge(this);
                *self = attrs;
                return;
            } else {
                panic!("params should be of type Params.")
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
            ParamTarget::Param => self.0.insert(attr.name, attr),
            ParamTarget::Class => match self.0.get_mut(&tags::class()) {
                Some(class) => {
                    let classes = class
                        .value
                        .get_mut::<String>()
                        .expect("Class param should be of type String.");
                    classes.push_str(" ");
                    classes.push_str(attr.name.into());
                    None
                }
                None => {
                    attr = Param::new(tags::class().into(), Variant::String(attr.name.into()));
                    self.0.insert(tags::class(), attr)
                }
            },
            ParamTarget::Style => match self.0.get_mut(&tags::styles()) {
                Some(styles) => {
                    let styles = styles
                        .value
                        .get_mut::<Params>()
                        .expect("Styles param should be of type Params.");
                    attr.target = ParamTarget::Param;
                    styles.add(attr);
                    None
                }
                None => {
                    let mut styles = Params::default();
                    attr.target = ParamTarget::Param;
                    styles.add(attr);
                    let attr = Param::new(tags::styles().into(), Variant::Params(styles));
                    self.0.insert(tags::styles(), attr)
                }
            },
        };
    }

    pub fn drain(&mut self) -> Drain<Tag, Param> {
        self.0.drain()
    }

    pub fn merge(&mut self, mut other: Self) {
        if let Some(other_classes) = other.0.remove(&tags::class()) {
            if let Some(self_classes) = self.0.get_mut(&tags::class()) {
                let self_class_string = self_classes
                    .value
                    .get_mut::<String>()
                    .expect("Class param should be of type String.");
                let other_class_string = other_classes
                    .value
                    .get::<String>()
                    .expect("Class param should be of type String.");
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
                    .get_mut::<Params>()
                    .expect("styles param should be of type Params");
                let other_styles_value = other_styles
                    .value
                    .get_mut::<Params>()
                    .expect("styles param should be of type Params");
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
        if let Some(mut styles) = self.drop::<Params>(tags::styles()) {
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
    pub fn get_variant(&self, key: Tag) -> Option<&Variant> {
        self.0.get(&key).map(|v| &v.value)
    }
    pub fn get_mut<T: 'static>(&mut self, key: Tag) -> Option<&mut T> {
        self.0.get_mut(&key).and_then(|v| v.value.get_mut::<T>())
    }
    pub fn drop<T: 'static>(&mut self, key: Tag) -> Option<T> {
        self.0.remove(&key).and_then(|mut a| a.take())
    }
    pub fn drop_variant(&mut self, key: Tag) -> Option<Variant> {
        self.0.remove(&key).map(|mut a| a.take_varint())
    }
    pub fn drop_or_default<T: 'static>(&mut self, key: Tag, default: T) -> T {
        if let Some(value) = self.drop(key) {
            value
        } else {
            default
        }
    }
    pub fn apply_commands(&mut self, for_param: Tag, commands: &mut EntityCommands) {
        if let Some(param_commands) = self.commands(for_param) {
            param_commands(commands)
        }
    }

    pub fn contains(&self, tag: Tag) -> bool {
        self.0.contains_key(&tag)
    }
}

pub struct InsertComponent<T: Component>(T);

pub fn component<T: Component>(component: T) -> InsertComponent<T> {
    InsertComponent(component)
}

impl From<i32> for Variant {
    fn from(v: i32) -> Self {
        Variant::Int(v)
    }
}

impl From<String> for Variant {
    fn from(v: String) -> Self {
        Variant::String(v)
    }
}

impl TryFrom<Variant> for String {
    type Error = String;
    fn try_from(variant: Variant) -> Result<Self, Self::Error> {
        variant
            .take::<String>()
            .ok_or("Can't cast variant to String".to_string())
    }
}

impl From<&str> for Variant {
    fn from(v: &str) -> Self {
        Variant::String(v.to_string())
    }
}

impl From<Entity> for Variant {
    fn from(v: Entity) -> Self {
        Variant::Entity(v)
    }
}

impl From<ApplyCommands> for Variant {
    fn from(commands: ApplyCommands) -> Self {
        Variant::Commands(commands)
    }
}

impl From<Params> for Variant {
    fn from(v: Params) -> Self {
        Variant::Params(v)
    }
}

impl<W: Component, T: BindValue> From<BindFrom<W, T>> for Variant {
    fn from(bind: BindFrom<W, T>) -> Self {
        Variant::BindFrom(bind.to_untyped())
    }
}

impl<R: Component, T: BindValue> From<BindTo<R, T>> for Variant {
    fn from(bind: BindTo<R, T>) -> Self {
        Variant::BindTo(bind.to_untyped())
    }
}

#[macro_export]
macro_rules! bindattr {
    ($ctx:ident, $key:ident:$typ:ident => $($target:tt)*) => {
        {
            let __elem = $ctx.entity();
            let __key = stringify!($key).as_tag();
            let __attr = $ctx.param(__key);
            let mut __value = Default::default();
            match __attr {
                Some($crate::Variant::BindFrom(__b)) => $ctx.commands().add(__b.to($crate::bind!(=> __elem, $($target)*))),
                Some($crate::Variant::BindTo(__b)) => $ctx.commands().add(__b.from($crate::bind!(<= __elem, $($target)*))),
                Some(__attr) => match $typ::try_from(__attr) {
                    Ok(__v) => __value = Some(__v),
                    Err(__err) => error!("Invalid value for '{}' param: {}", __key, __err)
                },
                _ => ()
            };
            __value
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_classes() {
        let mut attrs = Params::default();
        attrs.add(Param::new("class", "some-class".into()));
        assert_eq!(
            attrs.classes(),
            ["some-class".as_tag()].iter().cloned().collect()
        );
    }

    #[test]
    fn test_c_not_overrides_class() {
        let mut attrs = Params::default();
        attrs.add(Param::new("class", "class1 class2".into()));
        attrs.add(Param::new("c:some-other-class", Variant::Empty));
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
        let mut attrs = Params::default();
        attrs.add(Param::new("c:some-other-class", Variant::Empty));
        attrs.add(Param::new("class", "class1 class2".into()));
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
        let mut attrs = Params::default();
        attrs.add(Param::new("s:color", "black".into()));
        let styles = attrs.drop::<Params>("styles".as_tag());
        assert!(styles.is_some());
        let styles = styles.unwrap();
        assert_eq!(
            styles.get::<String>("color".as_tag()),
            Some(&"black".to_string())
        );
    }
}
