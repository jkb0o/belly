use bevy::prelude::*;
use eml::build::BuildPligin;
use eml::EmlPlugin;
use ess::{EssPlugin, StyleSheet, StyleSheetParser};
use input::ElementsInputPlugin;
use std::error::Error;
use std::fmt::Display;

pub mod element;
pub mod eml;
pub mod ess;
pub mod input;
pub mod relations;
pub mod tags;

pub struct ElementsCorePlugin;

pub use crate::element::ElementBundle;
pub use crate::element::ImageElementBundle;
pub use crate::element::TextElementBundle;
pub use crate::eml::build::ElementBuilder;
pub use crate::eml::build::ElementBuilderRegistry;
pub use crate::eml::build::ElementContext;
pub use crate::eml::build::ElementsBuilder;
pub use crate::eml::build::RegisterWidgetExtension;
pub use crate::eml::build::Widget;
pub use crate::eml::build::WidgetBuilder;
pub use crate::eml::content::ExpandElements;
pub use crate::eml::content::ExpandElementsExt;
pub use crate::eml::content::IntoContent;
pub use crate::eml::Param;
pub use crate::eml::Params;
pub use crate::eml::Variant;
pub use crate::ess::managed;
pub use crate::ess::CompoundProperty;
pub use crate::ess::PropertyValue;
pub use crate::ess::StylePropertyMethods;
pub use crate::ess::ToRectMap;
pub use crate::input::PointerInput;
pub use crate::input::PointerInputData;
pub use crate::relations::Connect;
pub use crate::relations::ConnectionTo;
pub use crate::relations::Signal;

// transformations
pub use crate::relations::bind::Prop;
pub use crate::relations::transform::TransformableTo;

// new bound system
pub use crate::relations::bind::TransformationError;
pub use crate::relations::bind::TransformationResult;
pub use crate::relations::transform::ColorTransformerExtension;

pub use element::Element;
pub use element::Elements;
pub use ess::Property;
pub use tagstr;
pub use tagstr::*;

use relations::RelationsPlugin;

pub mod prelude {}
pub mod build {
    pub use super::prelude::*;

    // macros
    pub use crate::compound_style_property;
    pub use crate::style_property;
    pub use crate::tag;

    // traits
    pub use crate::ess::RegisterProperty;
    pub use crate::ess::StylePropertyMethods;

    // structs
    pub use crate::ess::PropertyValue;
    pub use crate::ess::StyleProperty;
    pub use crate::Tag;
    pub use crate::Variant;
}

impl Plugin for ElementsCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Defaults::default())
            .add_plugin(ElementsInputPlugin)
            .add_plugin(RelationsPlugin)
            .add_plugin(BuildPligin)
            .add_plugin(EssPlugin)
            .add_plugin(EmlPlugin);

        // TODO: may be desabled with feature
        app.add_startup_system(setup_defaults);
    }
}

pub struct Widgets;
pub struct Transformers;

#[derive(Debug, PartialEq)]
pub enum ElementsError {
    /// An unsupported selector was found on a style sheet rule.
    UnsupportedSelector,
    /// An unsupported property was found on a style sheet rule.
    UnsupportedProperty(String),
    /// An invalid property value was found on a style sheet rule.
    InvalidPropertyValue(String),
    /// An invalid selector was found on a style sheet rule.
    InvalidSelector,
    /// An unexpected token was found on a style sheet rule.
    UnexpectedToken(String),
}

impl Error for ElementsError {}

impl Display for ElementsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementsError::UnsupportedSelector => {
                write!(f, "Unsupported selector")
            }
            ElementsError::UnsupportedProperty(p) => write!(f, "Unsupported property: {}", p),
            ElementsError::InvalidPropertyValue(p) => write!(f, "Invalid property value: {}", p),
            ElementsError::InvalidSelector => write!(f, "Invalid selector"),
            ElementsError::UnexpectedToken(t) => write!(f, "Unexpected token: {}", t),
        }
    }
}

#[derive(Default, Resource)]
pub struct Defaults {
    pub regular_font: Handle<Font>,
    pub italic_font: Handle<Font>,
    pub bold_font: Handle<Font>,
    pub bold_italic_font: Handle<Font>,
    pub style_sheet: Handle<StyleSheet>,
}

pub fn setup_defaults(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    mut defaults: ResMut<Defaults>,
    elements_registry: Res<ElementBuilderRegistry>,
    extractor: Res<ess::PropertyExtractor>,
    validator: Res<ess::PropertyTransformer>,
) {
    let font_bytes = include_bytes!("fonts/Exo2-ExtraLight.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.regular_font = font_handle;
    let font_bytes = include_bytes!("fonts/Exo2-ExtraLightItalic.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.italic_font = font_handle;
    let font_bytes = include_bytes!("fonts/Exo2-SemiBold.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.bold_font = font_handle;
    let font_bytes = include_bytes!("fonts/Exo2-SemiBoldItalic.ttf").to_vec();
    let font_asset = Font::try_from_bytes(font_bytes).unwrap();
    let font_handle = fonts.add(font_asset);
    defaults.bold_italic_font = font_handle;

    let parser = StyleSheetParser::new(validator.clone(), extractor.clone());
    let mut rules = parser.parse(
        r#"
            * {
                font: regular;
                color: #cfcfcf;
                font-size: 22px;
            }
        "#,
    );
    for rule in elements_registry.styles(parser) {
        rules.push(rule)
    }
    commands.add(StyleSheet::add_default(rules));
}
