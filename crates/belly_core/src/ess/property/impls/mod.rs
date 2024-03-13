pub mod flex_container;
pub mod flex_item;
pub mod grid;
pub mod layout_control;
pub mod size_constraints;
pub mod spacing;
pub mod stylebox;
pub mod text;

use super::parse;
use super::PropertyParser;
use super::StyleProperty;
use super::StylePropertyToken;
use crate::style_property;
use crate::ElementsError;
use bevy::prelude::*;

style_property! {
    #[doc = " TODO: write BacgroundColor description"]
    #[doc = " <!-- @property-category=General -->"]
    BackgroundColorProperty("background-color") {
        Default = "transparent";
        Item = Color;
        Components = &'static mut BackgroundColor;
        Filters = With<Node>;
        Parser = parse::ColorParser;
        Apply = |value, background, _assets, _commands, _entity| {
            if &background.0 != value {
                background.0 = *value;
            }
        };
    }
}

/// auto|$local|$global
pub struct OptionalZIndexParser;
impl PropertyParser<Option<ZIndex>> for OptionalZIndexParser {
    fn parse(value: &StyleProperty) -> Result<Option<ZIndex>, ElementsError> {
        let Some(token) = value.first() else {
            return Err(ElementsError::InvalidPropertyValue(format!(
                "Expected none|$local|$global, got nothing"
            )));
        };
        if let StylePropertyToken::Identifier(ident) = token {
            if ident == "auto" {
                return Ok(None);
            } else {
                return Err(ElementsError::InvalidPropertyValue(format!(
                    "Expected auto|$local|$global, got `{}`",
                    token.to_string()
                )));
            }
        }
        let StylePropertyToken::Dimension(num, unit) = token else {
            return Err(ElementsError::InvalidPropertyValue(format!(
                "Expected auto|$local|$global, got `{}`",
                token.to_string()
            )));
        };
        // Ok(None)
        match unit.as_str() {
            "l" => Ok(Some(ZIndex::Local(num.to_int()))),
            "g" => Ok(Some(ZIndex::Global(num.to_int()))),
            _ => Err(ElementsError::InvalidPropertyValue(format!(
                "Expected auto|$local|$global, got `{}`",
                token.to_string()
            ))),
        }
    }
}

style_property! {
    #[doc = " TODO: write ZIndex description"]
    #[doc = " <!-- @property-category=General -->"]
    ZIndexProperty("z-index") {
        Default = "auto";
        Item = Option<ZIndex>;
        Components = Option<&'static mut ZIndex>;
        Filters = With<Node>;
        Parser = OptionalZIndexParser;
        Apply = |value, zindex, _assets, commands, entity| {
            match (value, zindex) {
                (Some(index), Some(mut component)) => { *component = *index; }
                (Some(index), None) => { commands.entity(entity).insert(*index); }
                (None, None) => { }
                _ => { commands.entity(entity).remove::<ZIndex>(); }
            }
        };
    }
}
