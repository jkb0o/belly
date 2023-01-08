mod impls;
use std::{
    any::{type_name, Any, TypeId},
    fmt::Display,
    mem,
    str::FromStr,
};

use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    eml::Params,
    ess::{PropertyValue, StyleProperty},
    ElementsBuilder, StylePropertyMethods,
};
use std::fmt::Debug;

pub type ApplyCommands = Box<dyn FnOnce(&mut EntityCommands)>;

#[derive(Default)]
pub enum Variant {
    #[default]
    /// Empty Variant contains no value
    Undefined,
    Bool(bool),
    /// Contains String value.
    String(String),
    /// Contains Entity value
    Entity(Entity),
    /// Contains parsed styles tokens
    Style(StyleProperty),
    /// Contains extracted or transformed boxed property value
    /// ready to use with Property trait.
    Property(PropertyValue),
    Commands(ApplyCommands),
    Elements(ElementsBuilder),
    Params(Params),
    Boxed(Box<dyn Any>),
}

impl Debug for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::Undefined => write!(f, "Variant::Undefined"),
            Variant::Bool(v) => write!(f, "Variant::Bool({v})"),
            Variant::String(v) => write!(f, "Variant::String({:?})", v),
            Variant::Entity(v) => write!(f, "Variant::Entity({:?})", v),
            Variant::Style(v) => write!(f, "Variant::Style({})", v.to_string()),
            Variant::Property(_) => write!(f, "Variant::Property"),
            Variant::Params(v) => write!(f, "Variant::Params({:?})", v),
            Variant::Commands(_) => write!(f, "Variant::Commands"),
            Variant::Elements(_) => write!(f, "Variant::Elements"),
            Variant::Boxed(_) => write!(f, "Variant::Any"),
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
    pub fn string<T: ToString>(value: T) -> Variant {
        Variant::String(value.to_string())
    }
    pub fn boxed<T: 'static>(value: T) -> Variant {
        Variant::Boxed(Box::new(value))
    }
    pub fn style(value: StyleProperty) -> Variant {
        Variant::Style(value)
    }
    pub fn is<T: 'static>(&self) -> bool {
        match self {
            Variant::Undefined => false,
            Variant::Bool(_) => TypeId::of::<T>() == TypeId::of::<bool>(),
            Variant::String(_) => TypeId::of::<T>() == TypeId::of::<String>(),
            Variant::Entity(_) => TypeId::of::<T>() == TypeId::of::<Entity>(),
            Variant::Style(_) => TypeId::of::<T>() == TypeId::of::<StyleProperty>(),
            Variant::Property(_) => TypeId::of::<T>() == TypeId::of::<PropertyValue>(),
            Variant::Commands(_) => TypeId::of::<T>() == TypeId::of::<ApplyCommands>(),
            Variant::Elements(_) => TypeId::of::<T>() == TypeId::of::<ElementsBuilder>(),
            Variant::Params(_) => TypeId::of::<T>() == TypeId::of::<Params>(),
            Variant::Boxed(v) => v.is::<T>(),
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        match self {
            Variant::Undefined => None,
            Variant::Bool(v) => try_cast::<T, bool>(v),
            Variant::String(v) => try_cast::<T, String>(v),
            Variant::Entity(v) => try_cast::<T, Entity>(v),
            Variant::Style(v) => try_cast::<T, StyleProperty>(v),
            Variant::Property(v) => try_cast::<T, PropertyValue>(v),
            Variant::Commands(v) => try_cast::<T, ApplyCommands>(v),
            Variant::Elements(v) => try_cast::<T, ElementsBuilder>(v),
            Variant::Params(v) => try_cast::<T, Params>(v),
            Variant::Boxed(v) => v.downcast_ref::<T>(),
        }
    }
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        match self {
            Variant::Undefined => None,
            Variant::Bool(v) => try_cast_mut::<T, bool>(v),
            Variant::String(v) => try_cast_mut::<T, String>(v),
            Variant::Entity(v) => try_cast_mut::<T, Entity>(v),
            Variant::Style(v) => try_cast_mut::<T, StyleProperty>(v),
            Variant::Property(v) => try_cast_mut::<T, PropertyValue>(v),
            Variant::Commands(v) => try_cast_mut::<T, ApplyCommands>(v),
            Variant::Elements(v) => try_cast_mut::<T, ElementsBuilder>(v),
            Variant::Params(v) => try_cast_mut::<T, Params>(v),
            Variant::Boxed(v) => v.downcast_mut::<T>(),
        }
    }

    pub fn take<T: 'static>(self) -> Option<T> {
        match self {
            Variant::Undefined => None,
            Variant::Bool(v) => try_take::<T, bool>(v),
            Variant::String(v) => try_take::<T, String>(v),
            Variant::Entity(v) => try_take::<T, Entity>(v),
            Variant::Style(v) => try_take::<T, StyleProperty>(v),
            Variant::Property(v) => try_take::<T, PropertyValue>(v),
            Variant::Commands(v) => try_take::<T, ApplyCommands>(v),
            Variant::Elements(v) => try_take::<T, ElementsBuilder>(v),
            Variant::Params(v) => try_take::<T, Params>(v),
            Variant::Boxed(v) => match v.downcast::<T>() {
                Ok(v) => Some(*v),
                Err(v) => {
                    error!("Can't cast {:?} to {}", v, type_name::<T>());
                    None
                }
            },
        }
    }

    pub fn try_get<T: TryFrom<Variant, Error = impl std::fmt::Display>>(self) -> Option<T> {
        T::try_from(self).ok()
    }

    pub fn get_or<T: TryFrom<Variant, Error = impl std::fmt::Display>>(self, default: T) -> T {
        self.try_get().unwrap_or(default)
    }

    pub fn get_or_parse<T: FromStr<Err = impl Display> + 'static>(self) -> Result<T, String> {
        match self {
            Variant::String(s) => s.parse().map_err(|e| format!("{e}")),
            Variant::Boxed(b) => b
                .downcast::<T>()
                .map(|b| *b)
                .or_else(|b| {
                    b.downcast::<String>()
                        .map(|b| *b)
                        .or_else(|b| {
                            b.downcast::<&str>()
                                .map(|s| s.to_string())
                                .map_err(|_| format!("Invalid value for {}", type_name::<T>()))
                        })
                        .and_then(|s| s.parse::<T>().map_err(|e| format!("{e}")))
                })
                .map_err(|e| format!("{e}")),
            _ => Err(format!("Invalid value for {}", type_name::<T>())),
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
