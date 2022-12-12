mod impls;
use std::{
    any::{type_name, Any, TypeId},
    mem,
};

use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    params::Params,
    property::{PropertyValue, StyleProperty},
    relations::{BindFromUntyped, BindToUntyped},
    ElementsBuilder,
};
use std::fmt::Debug;

pub type ApplyCommands = Box<dyn FnOnce(&mut EntityCommands)>;

// pub trait Clonable: An

#[derive(Default)]
pub enum Variant {
    #[default]
    /// Empty Variant contains no value
    Empty,
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
    BindFrom(BindFromUntyped),
    BindTo(BindToUntyped),
    Boxed(Box<dyn Any>),
}

impl Debug for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::Empty => write!(f, "Variant::Empty"),
            // Variant::Int(v) => write!(f, "Variant::Int({:?})", v),
            // Variant::Real(v) => write!(f, "Variant::Int({:?})", v),
            Variant::String(v) => write!(f, "Variant::String({:?})", v),
            Variant::Entity(v) => write!(f, "Variant::Entity({:?})", v),
            Variant::Style(v) => write!(f, "Variant::Property({})", v.to_string()),
            Variant::Property(_) => write!(f, "Variant::Property"),
            Variant::Params(v) => write!(f, "Variant::Params({:?})", v),
            Variant::Commands(_) => write!(f, "Variant::Commands"),
            Variant::Elements(_) => write!(f, "Variant::Elements"),
            Variant::BindFrom(_) => write!(f, "Variant::BindFrom"),
            Variant::BindTo(_) => write!(f, "Variant::BindTo"),
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
            Variant::Empty => false,
            // Variant::Int(_) => num::is_int::<T>(),
            // Variant::Real(_) => num::is_real::<T>(),
            Variant::String(_) => TypeId::of::<T>() == TypeId::of::<String>(),
            Variant::Entity(_) => TypeId::of::<T>() == TypeId::of::<Entity>(),
            Variant::Style(_) => TypeId::of::<T>() == TypeId::of::<StyleProperty>(),
            Variant::Property(_) => TypeId::of::<T>() == TypeId::of::<PropertyValue>(),
            Variant::Commands(_) => TypeId::of::<T>() == TypeId::of::<ApplyCommands>(),
            Variant::Elements(_) => TypeId::of::<T>() == TypeId::of::<ElementsBuilder>(),
            Variant::Params(_) => TypeId::of::<T>() == TypeId::of::<Params>(),
            Variant::BindFrom(_) => TypeId::of::<T>() == TypeId::of::<BindFromUntyped>(),
            Variant::BindTo(_) => TypeId::of::<T>() == TypeId::of::<BindToUntyped>(),
            Variant::Boxed(v) => v.is::<T>(),
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        match self {
            Variant::Empty => None,
            // Variant::Int(v) => num::get_int_ref(v),
            // Variant::Real(v) => num::get_real_ref(v),
            Variant::String(v) => try_cast::<T, String>(v),
            Variant::Entity(v) => try_cast::<T, Entity>(v),
            Variant::Style(v) => try_cast::<T, StyleProperty>(v),
            Variant::Property(v) => try_cast::<T, PropertyValue>(v),
            Variant::Commands(v) => try_cast::<T, ApplyCommands>(v),
            Variant::Elements(v) => try_cast::<T, ElementsBuilder>(v),
            Variant::Params(v) => try_cast::<T, Params>(v),
            Variant::BindFrom(v) => try_cast::<T, BindFromUntyped>(v),
            Variant::BindTo(v) => try_cast::<T, BindToUntyped>(v),
            Variant::Boxed(v) => v.downcast_ref::<T>(),
        }
    }
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        match self {
            Variant::Empty => None,
            // Variant::Int(v) => num::get_int_mut(v),
            // Variant::Real(v) => num::get_real_mut(v),
            Variant::String(v) => try_cast_mut::<T, String>(v),
            Variant::Entity(v) => try_cast_mut::<T, Entity>(v),
            Variant::Style(v) => try_cast_mut::<T, StyleProperty>(v),
            Variant::Property(v) => try_cast_mut::<T, PropertyValue>(v),
            Variant::Commands(v) => try_cast_mut::<T, ApplyCommands>(v),
            Variant::Elements(v) => try_cast_mut::<T, ElementsBuilder>(v),
            Variant::Params(v) => try_cast_mut::<T, Params>(v),
            Variant::BindFrom(v) => try_cast_mut::<T, BindFromUntyped>(v),
            Variant::BindTo(v) => try_cast_mut::<T, BindToUntyped>(v),
            Variant::Boxed(v) => v.downcast_mut::<T>(),
        }
    }

    pub fn take<T: 'static>(self) -> Option<T> {
        match self {
            Variant::Empty => None,
            // Variant::Int(v) => num::get_int(v),
            // Variant::Real(v) => num::get_real(v),
            Variant::String(v) => try_take::<T, String>(v),
            Variant::Entity(v) => try_take::<T, Entity>(v),
            Variant::Style(v) => try_take::<T, StyleProperty>(v),
            Variant::Property(v) => try_take::<T, PropertyValue>(v),
            Variant::Commands(v) => try_take::<T, ApplyCommands>(v),
            Variant::Elements(v) => try_take::<T, ElementsBuilder>(v),
            Variant::Params(v) => try_take::<T, Params>(v),
            Variant::BindFrom(v) => try_take::<T, BindFromUntyped>(v),
            Variant::BindTo(v) => try_take::<T, BindToUntyped>(v),
            Variant::Boxed(v) => match v.downcast::<T>() {
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
