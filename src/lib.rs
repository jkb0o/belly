use bevy::prelude::*;

// plugin
use belly_core::ElementsCorePlugin;
use belly_widgets::WidgetsPlugin;

// bundles
pub use belly_core::ElementBundle;
pub use belly_core::ImageElementBundle;
pub use belly_core::TextElementBundle;

// structs
pub use belly_core::eml::asset::EmlAsset;
pub use belly_core::eml::asset::EmlScene;
pub use belly_core::ess::StyleSheet;

// macros
pub use belly_core::bind;
pub use belly_core::connect;
pub use belly_core::from;
pub use belly_core::to;
pub use belly_macro::eml;

// traits
pub use belly_core::eml::build::WidgetBuilder;
pub use belly_core::eml::content::IntoContent;
pub use belly_core::property::colors::ColorFromHexExtension;
pub use belly_core::relations::transform::ColorTransformerExtension;
pub use belly_core::ExpandElementsExt;
pub use belly_core::WithElements;

// widgets
pub use belly_widgets::prelude::*;

pub struct ElementsPlugin;
impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ElementsCorePlugin);
        app.add_plugin(WidgetsPlugin);
    }
}

pub mod build {
    pub use super::*;
    pub use belly_core::ElementBuilder;
    pub use belly_core::ElementContext;
    pub use belly_core::RegisterWidgetExtension;
    pub use belly_core::Widgets;
    pub use belly_macro::widget;
}
