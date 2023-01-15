use std::{any::Any, marker::PhantomData};

use crate::ElementsError;
use bevy::prelude::*;

use super::{colors, PropertyParser, StyleProperty, StylePropertyMethods, StylePropertyToken};

pub fn identifier<S>(prop: &StyleProperty) -> Result<S, ElementsError>
where
    S: for<'a> TryFrom<&'a StyleProperty, Error = ElementsError> + Default + Any + Send + Sync,
{
    S::try_from(prop)
}
pub struct IdentifierParser<S>(PhantomData<S>);
impl<S> PropertyParser<S> for IdentifierParser<S>
where
    S: for<'a> TryFrom<&'a StyleProperty, Error = ElementsError> + Default + Any + Send + Sync,
{
    fn parse(value: &StyleProperty) -> Result<S, ElementsError> {
        identifier::<S>(value)
    }
}

// }

pub fn val(prop: &StyleProperty) -> Result<Val, ElementsError> {
    let Some(prop) = prop.first() else {
        return Err(ElementsError::InvalidPropertyValue(format!("Expected $val, found nothing")))
    };
    match prop {
        StylePropertyToken::Percentage(val) => Ok(Val::Percent(val.into())),
        StylePropertyToken::Dimension(val, unit) if unit.as_str() == "px" => {
            Ok(Val::Px(val.into()))
        }
        StylePropertyToken::Identifier(val) if val.as_str() == "auto" => Ok(Val::Auto),
        StylePropertyToken::Identifier(val) if val.as_str() == "undefined" => Ok(Val::Undefined),
        p => Err(ElementsError::InvalidPropertyValue(format!(
            "Expected $val, got `{}`",
            p.to_string()
        ))),
    }
}

pub struct ValParser;
impl PropertyParser<Val> for ValParser {
    fn parse(value: &StyleProperty) -> Result<Val, ElementsError> {
        val(value)
    }
}

pub fn rect(prop: &StyleProperty) -> Result<UiRect, ElementsError> {
    match prop.len() {
        1 => prop[0].val().map(UiRect::all),
        2 => {
            let top_bottom = prop[0].val()?;
            let left_right = prop[1].val()?;
            Ok(UiRect::new(left_right, left_right, top_bottom, top_bottom))
        }
        3 => {
            let top = prop[0].val()?;
            let left_right = prop[1].val()?;
            let bottom = prop[2].val()?;
            Ok(UiRect::new(left_right, left_right, top, bottom))
        }
        4 => {
            let top = prop[0].val()?;
            let right = prop[1].val()?;
            let bottom = prop[2].val()?;
            let left = prop[3].val()?;
            Ok(UiRect::new(left, right, top, bottom))
        }
        _ => Err(ElementsError::InvalidPropertyValue(format!(
            "Expected $rect, got `{}`",
            prop.to_string()
        ))),
    }
}

pub struct RectParser;
impl PropertyParser<UiRect> for RectParser {
    fn parse(value: &StyleProperty) -> Result<UiRect, ElementsError> {
        rect(value)
    }
}

pub fn color(prop: &StyleProperty) -> Result<Color, ElementsError> {
    if prop.len() == 0 {
        return Err(ElementsError::InvalidPropertyValue(format!(
            "Expected $color, got nothing"
        )));
    }
    match &prop[0] {
        StylePropertyToken::Identifier(name) => colors::parse_named_color(name.as_str())
            .ok_or_else(|| {
                ElementsError::InvalidPropertyValue(format!("Unknown color name `{name}`"))
            }),
        StylePropertyToken::Hash(hash) => colors::parse_hex_color(hash.as_str()),
        prop => {
            return Err(ElementsError::InvalidPropertyValue(format!(
                "Expected $color, got `{}`",
                prop.to_string()
            )))
        }
    }
}
pub struct ColorParser;
impl PropertyParser<Color> for ColorParser {
    fn parse(value: &StyleProperty) -> Result<Color, ElementsError> {
        color(value)
    }
}

pub fn string(prop: &StyleProperty) -> Result<String, ElementsError> {
    let Some(token) = prop.first() else {
        return Err(ElementsError::InvalidPropertyValue(format!("Expected $string, got nothing")));
    };
    match token {
        StylePropertyToken::String(id) => Ok(id.clone()),
        e => Err(ElementsError::InvalidPropertyValue(format!(
            "Expected $string, got `{}`",
            e.to_string()
        ))),
    }
}
pub struct StringParser;
impl PropertyParser<String> for StringParser {
    fn parse(value: &StyleProperty) -> Result<String, ElementsError> {
        string(value)
    }
}

pub fn optional_string(prop: &StyleProperty) -> Result<Option<String>, ElementsError> {
    let Some(token) = prop.first() else {
        return Err(ElementsError::InvalidPropertyValue(format!("Expected none|$string, got nothing")));
    };
    match token {
        StylePropertyToken::Identifier(ident) if ident == "none" => Ok(None),
        StylePropertyToken::String(id) => Ok(Some(id.clone())),
        e => Err(ElementsError::InvalidPropertyValue(format!(
            "Expected none|$string, got `{}`",
            e.to_string()
        ))),
    }
}
pub struct OptionalStringParser;
impl PropertyParser<Option<String>> for OptionalStringParser {
    fn parse(value: &StyleProperty) -> Result<Option<String>, ElementsError> {
        optional_string(value)
    }
}

pub fn num(prop: &StyleProperty) -> Result<f32, ElementsError> {
    let Some(prop) = prop.first() else {
        return Err(ElementsError::InvalidPropertyValue(format!("Expected $num, got nothing")))
    };
    match prop {
        StylePropertyToken::Percentage(val)
        | StylePropertyToken::Dimension(val, _)
        | StylePropertyToken::Number(val) => Ok(val.into()),
        p => Err(ElementsError::InvalidPropertyValue(format!(
            "Expected $num, got `{}`",
            p.to_string()
        ))),
    }
}
pub struct NumParser;
impl PropertyParser<f32> for NumParser {
    fn parse(value: &StyleProperty) -> Result<f32, ElementsError> {
        num(value)
    }
}

pub fn optional_num(prop: &StyleProperty) -> Result<Option<f32>, ElementsError> {
    let Some(prop) = prop.first() else {
        return Err(ElementsError::InvalidPropertyValue(format!("Expected none|$num, got nothing")))
    };
    match prop {
        StylePropertyToken::Percentage(val)
        | StylePropertyToken::Dimension(val, _)
        | StylePropertyToken::Number(val) => Ok(Some(val.into())),
        StylePropertyToken::Identifier(ident) => match ident.as_str() {
            "none" => Ok(None),
            ident => Err(ElementsError::InvalidPropertyValue(format!(
                "Expected none|$num, got `{ident}`"
            ))),
        },
        e => Err(ElementsError::InvalidPropertyValue(format!(
            "Expected none|$num, got `{}`",
            e.to_string()
        ))),
    }
}
pub struct OptionalNumParser;
impl PropertyParser<Option<f32>> for OptionalNumParser {
    fn parse(value: &StyleProperty) -> Result<Option<f32>, ElementsError> {
        optional_num(value)
    }
}
