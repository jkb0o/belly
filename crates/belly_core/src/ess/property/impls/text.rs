use super::parse;
use crate::ess::defaults::Defaults;
use crate::ess::PropertyParser;
use crate::ess::StyleProperty;
use crate::ess::StylePropertyToken;
use crate::style_property;
use crate::ElementsError;
use bevy::prelude::*;

#[derive(Default, Clone)]
pub enum FontPath {
    #[default]
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Custom(String),
}

/// regular|bold|italic|bold-italic|$string
pub struct FontParser;
impl PropertyParser<FontPath> for FontParser {
    fn parse(prop: &StyleProperty) -> Result<FontPath, ElementsError> {
        let Some(token) = prop.first() else {
            return Err(ElementsError::InvalidPropertyValue(format!(
                "Expected regular|bold|italic|bold-italic|$string, got nothing"
            )));
        };
        match token {
            StylePropertyToken::String(id) => Ok(FontPath::Custom(id.clone())),
            StylePropertyToken::Identifier(ident) => match ident.as_str() {
                "regular" => Ok(FontPath::Regular),
                "bold" => Ok(FontPath::Bold),
                "italic" => Ok(FontPath::Italic),
                "bold-italic" => Ok(FontPath::BoldItalic),
                ident => {
                    return Err(ElementsError::InvalidPropertyValue(format!(
                        "Expected regular|bold|italic|bold-italic|$string, got `{ident}`"
                    )))
                }
            },
            ident => Err(ElementsError::InvalidPropertyValue(format!(
                "Expected regulart|bold|italic|bold-italic|$string, got `{}`",
                ident.to_string()
            ))),
        }
    }
}

style_property! {
    #[doc = " TODO: wtite FontProperty description"]
    #[doc = " <!-- @property-category=Text -->"]
    FontProperty("font") {
        Default = "regular";
        Item = FontPath;
        Components = &'static mut Text;
        Filters = With<Node>;
        AffectsVirtual = true;
        Parser = FontParser;
        Apply = |value, text, assets, commands, entity| {
            if let FontPath::Custom(path) = value {
                text
                    .sections
                    .iter_mut()
                    .for_each(|section| section.style.font = assets.load(path));
            } else {
                let path = value.clone();
                commands.add(move |world: &mut World| {
                    let defaults = world.resource::<Defaults>();
                    let font = match path {
                        FontPath::Regular => defaults.regular_font.clone(),
                        FontPath::Italic => defaults.italic_font.clone(),
                        FontPath::Bold => defaults.bold_font.clone(),
                        FontPath::BoldItalic => defaults.bold_italic_font.clone(),
                        _ => defaults.regular_font.clone(),
                    };
                    world
                        .entity_mut(entity)
                        .get_mut::<Text>()
                        .unwrap()
                        .sections
                        .iter_mut()
                        .for_each(|section| section.style.font = font.clone());
                });
            }
        };
    }
}

style_property! {
    #[doc = " TODO: remove depricate ColorProperty"]
    #[doc = " <!-- @property-category=Text -->"]
    ColorProperty("color") {
        Default = "#cfcfcf";
        Item = Color;
        Components = &'static mut Text;
        Filters = With<Node>;
        AffectsVirtual = true;
        Parser = parse::ColorParser;
        Apply = |value, text, _assets, _commands, _entity| {
            // TODO: mark it deprecated
            text
                .sections
                .iter_mut()
                .for_each(|section| section.style.color = *value);
        };
    }
}

style_property! {
    #[doc = " TODO: write FontSizeProperty description"]
    #[doc = " <!-- @property-category=Text -->"]
    FontSizeProperty("font-size") {
        Default = "24";
        Item = f32;
        Components = &'static mut Text;
        Filters = With<Node>;
        AffectsVirtual = true;
        Parser = parse::NumParser;
        Apply = |value, text, _assets, _commands, _entity| {
            text
                .sections
                .iter_mut()
                .for_each(|section| section.style.font_size = *value);
        };
    }
}
//     /// Applies the `vertical-align` property on [`TextAlignment::vertical`](`TextAlignment`) property of matched [`Text`] components.
//     #[derive(Default)]
//     pub(crate) struct VerticalAlignProperty;

//     impl Property for VerticalAlignProperty {
//         // Using Option since Cache must impl Default, which VerticalAlign doesn't
//         type Item = Option<VerticalAlign>;
//         type Components = &'static mut Text;
//         type Filters = With<Node>;

//         fn name() -> Tag {
//             tag!("vertical-align")
//         }

//         fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
//             if let Ok(ident) = values.identifier() {
//                 match ident {
//                     "top" => return Ok(Some(VerticalAlign::Top)),
//                     "center" => return Ok(Some(VerticalAlign::Center)),
//                     "bottom" => return Ok(Some(VerticalAlign::Bottom)),
//                     _ => (),
//                 }
//             }
//             Err(ElementsError::InvalidPropertyValue(
//                 Self::name().to_string(),
//             ))
//         }

//         fn apply<'w>(
//             cache: &Self::Item,
//             mut components: QueryItem<Self::Components>,
//             _asset_server: &AssetServer,
//             _commands: &mut Commands,
//             _entity: Entity,
//         ) {
//             components.alignment.vertical = cache.expect("Should always have a inner value");
//         }
//     }

//     /// Applies the `text-align` property on [`TextAlignment::horizontal`](`TextAlignment`) property of matched [`Text`] components.
//     #[derive(Default)]
//     pub(crate) struct HorizontalAlignProperty;

//     impl Property for HorizontalAlignProperty {
//         // Using Option since Cache must impl Default, which HorizontalAlign doesn't
//         type Item = Option<HorizontalAlign>;
//         type Components = &'static mut Text;
//         type Filters = With<Node>;

//         fn name() -> Tag {
//             tag!("text-align")
//         }

//         fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
//             if let Ok(ident) = values.identifier() {
//                 match ident {
//                     "left" => return Ok(Some(HorizontalAlign::Left)),
//                     "center" => return Ok(Some(HorizontalAlign::Center)),
//                     "right" => return Ok(Some(HorizontalAlign::Right)),
//                     _ => (),
//                 }
//             }
//             Err(ElementsError::InvalidPropertyValue(
//                 Self::name().to_string(),
//             ))
//         }

//         fn apply<'w>(
//             cache: &Self::Item,
//             mut components: QueryItem<Self::Components>,
//             _asset_server: &AssetServer,
//             _commands: &mut Commands,
//             _entity: Entity,
//         ) {
//             components.alignment.horizontal = cache.expect("Should always have a inner value");
//         }
//     }

//     /// Apply a custom `text-content` which updates [`TextSection::value`](`TextSection`) of all sections on matched [`Text`] components
//     #[derive(Default)]
//     pub(crate) struct TextContentProperty;

//     impl Property for TextContentProperty {
//         type Item = String;
//         type Components = &'static mut Text;
//         type Filters = With<Node>;

//         fn name() -> Tag {
//             tag!("text-content")
//         }

//         fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
//             values.string()
//         }

//         fn apply<'w>(
//             cache: &Self::Item,
//             mut components: QueryItem<Self::Components>,
//             _asset_server: &AssetServer,
//             _commands: &mut Commands,
//             _entity: Entity,
//         ) {
//             components
//                 .sections
//                 .iter_mut()
//                 // TODO: Maybe change this so each line break is a new section
//                 .for_each(|section| section.value = cache.clone());
//         }
//     }
