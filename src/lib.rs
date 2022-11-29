use bevy::prelude::*;

// plugin
use bevy_elements_core::ElementsCorePlugin;
use bevy_elements_widgets::WidgetsPlugin;

// structs
pub use bevy_elements_core::ess::StyleSheet;

// macros
pub use bevy_elements_core::bind;
pub use bevy_elements_macro::eml;

// traits
pub use bevy_elements_core::ExpandElementsExt;
pub use bevy_elements_core::WithElements;
pub use bevy_elements_core::builders::Widget;
pub use bevy_elements_core::context::IntoContent;

// widgets
pub use bevy_elements_widgets::prelude::*;

pub struct ElementsPlugin;
impl Plugin for ElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ElementsCorePlugin);
        app.add_plugin(WidgetsPlugin);
    }
}

pub mod build {
    pub use super::*;
    pub use bevy_elements_core::RegisterElementBuilder;
    pub use bevy_elements_core::BuildingContext;
}
