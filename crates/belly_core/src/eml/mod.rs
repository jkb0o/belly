pub mod asset;
pub mod build;
pub mod content;
pub mod params;
pub mod parse;
pub mod variant;
pub use self::build::*;
pub use self::params::*;
pub use self::variant::*;
use crate::ess::{PropertyExtractor, PropertyTransformer};
use asset::{update_eml_scene, EmlAsset, EmlLoader};
use bevy::prelude::*;

#[derive(Default)]
pub struct EmlPlugin;

impl Plugin for EmlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<EmlAsset>();
        let extractor = app
            .world
            .get_resource_or_insert_with(PropertyExtractor::default)
            .clone();
        let validator = app
            .world
            .get_resource_or_insert_with(PropertyTransformer::default)
            .clone();
        let registry = app
            .world
            .get_resource_or_insert_with(WidgetRegistry::default)
            .clone();

        app.register_asset_loader(EmlLoader {
            transformer: validator,
            extractor,
            registry,
        });
        app.add_systems(Update, update_eml_scene);
    }
}
