use crate::{
    params::Params,
    relations::{BindFrom, BindTo, BindValue},
};
use bevy::prelude::*;

use super::{ApplyCommands, Variant};

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

impl From<JustifyContent> for Variant {
    fn from(value: JustifyContent) -> Self {
        Variant::Boxed(Box::new(value))
    }
}

pub trait TryParse: Sized {
    type Error;
    fn try_parse(value: &str) -> Result<Self, Self::Error>;
}

impl TryParse for JustifyContent {
    type Error = String;
    fn try_parse(value: &str) -> Result<Self, Self::Error> {
        match value {
            "flex-start" => Ok(JustifyContent::FlexStart),
            "flex-end" => Ok(JustifyContent::FlexEnd),
            "center" => Ok(JustifyContent::Center),
            "space-between" => Ok(JustifyContent::SpaceBetween),
            "space-around" => Ok(JustifyContent::SpaceAround),
            "space-evenly" => Ok(JustifyContent::SpaceEvenly),
            invalid => Err(format!("Can't parse `{}` as JustifyContent", invalid)),
        }
    }
}

impl TryFrom<&Variant> for JustifyContent {
    type Error = String;
    fn try_from(value: &Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::String(s) => JustifyContent::try_parse(s),
            variant => {
                if let Some(value) = variant.get::<JustifyContent>() {
                    Ok(value.clone())
                } else {
                    Err("Invalid value for JustifyContent".to_string())
                }
            }
        }
    }
}
