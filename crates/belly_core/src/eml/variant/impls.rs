use std::any::type_name;

use crate::{
    eml::Params,
    ess::{ColorFromHexExtension, StyleProperty, StylePropertyMethods},
    ElementsError,
};
use bevy::{asset::Asset, prelude::*};

use super::{ApplyCommands, Variant};

impl From<String> for Variant {
    fn from(v: String) -> Self {
        Variant::String(v)
    }
}

impl TryFrom<Variant> for String {
    type Error = String;
    fn try_from(variant: Variant) -> Result<Self, Self::Error> {
        match variant {
            Variant::Undefined => Ok("".to_string()),
            Variant::String(value) => Ok(value),
            Variant::Boxed(b) => b
                .downcast::<String>()
                .map(|b| *b)
                .or_else(|b| b.downcast::<&str>().map(|s| s.to_string()))
                .map_err(|_| "Not a valid String".to_string()),
            _ => Err("Not a valid String".to_string()),
        }
    }
}

impl From<bool> for Variant {
    fn from(v: bool) -> Self {
        Variant::Bool(v)
    }
}

impl TryFrom<Variant> for f32 {
    type Error = String;
    fn try_from(variant: Variant) -> Result<Self, Self::Error> {
        match variant {
            Variant::String(s) => s.parse().map_err(|e| format!("Can't parse {e} as f32")),
            variant => variant
                .take::<f32>()
                .ok_or_else(|| format!("Can't cast Variant to f32")),
        }
    }
}

impl From<f32> for Variant {
    fn from(v: f32) -> Self {
        Variant::boxed(v)
    }
}

impl TryFrom<Variant> for u8 {
    type Error = String;
    fn try_from(variant: Variant) -> Result<Self, Self::Error> {
        match variant {
            Variant::String(s) => s.parse().map_err(|e| format!("Can't parse {e} as u8")),
            variant => variant
                .take::<u8>()
                .ok_or_else(|| format!("Can't cast Variant to u8")),
        }
    }
}

impl From<u8> for Variant {
    fn from(v: u8) -> Self {
        Variant::boxed(v)
    }
}

impl TryFrom<Variant> for bool {
    type Error = String;
    fn try_from(variant: Variant) -> Result<Self, Self::Error> {
        match variant {
            Variant::Undefined => Ok(false),
            Variant::Bool(v) => Ok(v),
            Variant::String(s) if &s == "yes" => Ok(true),
            Variant::String(s) if &s == "Yes" => Ok(true),
            Variant::String(s) if &s == "YES" => Ok(true),
            Variant::String(s) if &s == "true" => Ok(true),
            Variant::String(s) if &s == "True" => Ok(true),
            Variant::String(s) if &s == "TRUE" => Ok(true),
            Variant::String(s) if &s == "no" => Ok(false),
            Variant::String(s) if &s == "No" => Ok(false),
            Variant::String(s) if &s == "NO" => Ok(false),
            Variant::String(s) if &s == "false" => Ok(false),
            Variant::String(s) if &s == "false" => Ok(false),
            Variant::String(s) if &s == "FALSE" => Ok(false),
            Variant::Boxed(b) => b
                .downcast::<bool>()
                .map(|v| *v)
                .map_err(|e| format!("Can't extract bool from {:?}", e)),
            invalid => Err(format!("Can't extract bool from {:?}", invalid)),
        }
    }
}

impl From<&str> for Variant {
    fn from(v: &str) -> Self {
        Variant::String(v.to_string())
    }
}

impl From<&&str> for Variant {
    fn from(v: &&str) -> Self {
        Variant::String(v.to_string())
    }
}

impl From<Entity> for Variant {
    fn from(v: Entity) -> Self {
        Variant::Entity(v)
    }
}

impl TryFrom<Variant> for Entity {
    type Error = String;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Entity(e) => Ok(e),
            Variant::Boxed(v) => v
                .downcast::<Entity>()
                .map(|v| *v)
                .map_err(|e| format!("Can't extract Entity from {:?}", e)),
            e => Err(format!("Can't extract Entity from {:?}", e)),
        }
    }
}

impl From<Color> for Variant {
    fn from(v: Color) -> Self {
        Variant::boxed(v)
    }
}

impl TryFrom<Variant> for Color {
    type Error = String;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::String(s) => Color::try_from_hex(s),
            Variant::Boxed(v) => v
                .downcast::<Color>()
                .map(|b| *b)
                .map_err(|e| format!("Can't extract Color from {:?}", e)),
            e => Err(format!("Can't extract Color from {:?}", e)),
        }
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

impl From<Val> for Variant {
    fn from(val: Val) -> Self {
        Variant::boxed(val)
    }
}

impl TryFrom<Variant> for UiRect {
    type Error = ElementsError;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        let rect = match value {
            Variant::String(unparsed) => {
                StyleProperty::try_from(unparsed).and_then(|prop| prop.rect())?
            }
            Variant::Style(prop) => prop.rect()?,
            variant => variant
                .take::<UiRect>()
                .ok_or(ElementsError::InvalidPropertyValue(format!(
                    "Can't extract rect from variant"
                )))?,
        };
        Ok(rect)
    }
}

impl<T: Asset> From<Handle<T>> for Variant {
    fn from(value: Handle<T>) -> Self {
        Variant::boxed(value)
    }
}

impl<T: Asset> TryFrom<Variant> for Handle<T> {
    type Error = String;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::Boxed(v) if v.is::<Handle<T>>() => Ok(*v.downcast::<Handle<T>>().unwrap()),
            e => Err(format!(
                "Can't extract Handle<{}> from variant '{e:?}",
                type_name::<T>()
            )),
        }
    }
}
