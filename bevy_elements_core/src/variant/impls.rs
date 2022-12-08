use crate::{
    params::Params,
    relations::{BindFrom, BindTo, BindValue},
};
use bevy::prelude::*;

use super::{ApplyCommands, Variant};
impl From<i32> for Variant {
    fn from(v: i32) -> Self {
        Variant::Int(v as isize)
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
