pub mod element;
pub mod eml;
pub mod ess;
pub mod input;
pub mod relations;
pub mod tags;
use crate::eml::EmlPlugin;
use crate::ess::EssPlugin;
use crate::input::ElementsInputPlugin;
use crate::relations::RelationsPlugin;
use bevy::prelude::*;
use element::ElementsPlugin;
use eml::BuildPlugin;
pub use tagstr;
pub use tagstr::*;

pub mod prelude {
    // funcs
    pub use crate::ess::managed;

    // macro
    pub use crate::bind;
    pub use crate::from;
    pub use crate::to;

    // traits
    pub use crate::eml::content::ExpandElementsExt;
    pub use crate::eml::content::IntoContent;
    pub use crate::eml::Widget;
    pub use crate::ess::ColorFromHexExtension;
    pub use crate::relations::connect::ConnectCommandsExtension;

    // structs
    pub use crate::element::Element;
    pub use crate::element::Elements;
    pub use crate::eml::asset::EmlAsset;
    pub use crate::eml::asset::EmlScene;
    pub use crate::ess::StyleSheet;
    pub use crate::relations::connect::Connect;
    pub use crate::relations::connect::EventSource;
    pub use crate::relations::EventContext;
}

pub mod build {
    pub use super::prelude::*;

    // macros
    pub use crate::compound_style_property;
    pub use crate::style_property;
    pub use crate::tag;

    // traits
    pub use crate::eml::FromWorldAndParams;
    pub use crate::eml::RegisterWidget;
    pub use crate::ess::RegisterProperty;
    pub use crate::ess::StylePropertyMethods;
    pub use crate::relations::bind::AsTransformer;
    pub use crate::relations::bind::TransformationResult;
    pub use crate::relations::props::impls::OptionProperties;
    pub use crate::relations::props::GetProperties;

    // structs
    pub use crate::element::ElementBundle;
    pub use crate::element::TextElementBundle;
    pub use crate::eml::Variant;
    pub use crate::eml::WidgetContext;
    pub use crate::eml::WidgetData;
    pub use crate::ess::PropertyValue;
    pub use crate::ess::StyleProperty;
    pub use crate::input::PointerInput;
    pub use crate::input::PointerInputData;
    pub use crate::relations::props::Prop;
    pub use crate::relations::Handler;
    pub use crate::Tag;
}

pub struct ElementsCorePlugin;
impl Plugin for ElementsCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ElementsPlugin)
            .add_plugins(ElementsInputPlugin)
            .add_plugins(RelationsPlugin)
            .add_plugins(BuildPlugin)
            .add_plugins(EssPlugin)
            .add_plugins(EmlPlugin);
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
    EndOfInput,
}

impl std::error::Error for ElementsError {}

impl std::fmt::Display for ElementsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementsError::UnsupportedSelector => {
                write!(f, "Unsupported selector")
            }
            ElementsError::UnsupportedProperty(p) => write!(f, "Unsupported property: {}", p),
            ElementsError::InvalidPropertyValue(p) => write!(f, "Invalid property value: {}", p),
            ElementsError::InvalidSelector => write!(f, "Invalid selector"),
            ElementsError::UnexpectedToken(t) => write!(f, "Unexpected token: {}", t),
            ElementsError::EndOfInput => write!(f, "Unexpected end of input"),
        }
    }
}
