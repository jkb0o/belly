pub mod element;
pub mod eml;
pub mod ess;
pub mod input;
pub mod relations;
pub mod tags;
use crate::eml::build::BuildPligin;
use crate::eml::EmlPlugin;
use crate::ess::EssPlugin;
use crate::input::ElementsInputPlugin;
use crate::relations::RelationsPlugin;
use bevy::prelude::*;
pub use tagstr;
pub use tagstr::*;

pub mod prelude {
    // funcs
    pub use crate::ess::managed;

    // macro
    pub use crate::bind;
    pub use crate::connect;
    pub use crate::from;
    pub use crate::to;

    // traits
    pub use crate::eml::content::ExpandElementsExt;
    pub use crate::eml::content::IntoContent;
    pub use crate::ess::ColorFromHexExtension;
    pub use crate::relations::transform::ColorTransformerExtension;
    pub use crate::relations::Signal;

    // structs
    pub use crate::element::Element;
    pub use crate::element::Elements;
    pub use crate::eml::asset::EmlAsset;
    pub use crate::eml::asset::EmlScene;
    pub use crate::ess::StyleSheet;
    pub use crate::relations::ConnectionEntityContext;
    pub use crate::relations::ConnectionGeneralContext;
}
pub mod build {
    pub use super::prelude::*;

    // macros
    pub use crate::compound_style_property;
    pub use crate::style_property;
    pub use crate::tag;

    // traits
    pub use crate::eml::build::FromWorldAndParam;
    pub use crate::eml::build::RegisterWidgetExtension;
    pub use crate::eml::build::Widget;
    pub use crate::eml::build::WidgetBuilder;
    pub use crate::ess::RegisterProperty;
    pub use crate::ess::StylePropertyMethods;
    pub use crate::relations::bind::AsTransformer;
    pub use crate::relations::bind::TransformationResult;
    pub use crate::relations::transform::TransformableTo;

    // structs
    pub use crate::element::ElementBundle;
    pub use crate::element::TextElementBundle;
    pub use crate::eml::build::ElementBuilder;
    pub use crate::eml::build::ElementContext;
    pub use crate::eml::build::ElementContextData;
    pub use crate::eml::build::ElementsBuilder;
    pub use crate::eml::Variant;
    pub use crate::ess::PropertyValue;
    pub use crate::ess::StyleProperty;
    pub use crate::input::PointerInput;
    pub use crate::input::PointerInputData;
    pub use crate::relations::bind::Prop;
    pub use crate::relations::ConnectionTo;
    pub use crate::Tag;
}

pub struct ElementsCorePlugin;
impl Plugin for ElementsCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ElementsInputPlugin)
            .add_plugin(RelationsPlugin)
            .add_plugin(BuildPligin)
            .add_plugin(EssPlugin)
            .add_plugin(EmlPlugin);
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
        }
    }
}
