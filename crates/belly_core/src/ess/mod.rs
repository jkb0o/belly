mod defaults;
mod parser;
pub mod property;
mod selector;

pub use self::parser::StyleSheetParser;
use crate::{element::Elements, ess::defaults::Defaults};
use anyhow::Error;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt},
    ecs::system::Command,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    utils::{hashbrown::hash_map::Keys, BoxedFuture, HashMap},
};
pub use property::*;
pub use selector::*;
use smallvec::SmallVec;
use std::ops::Deref;
use tagstr::Tag;
use thiserror::Error;

#[derive(Default)]
pub struct EssPlugin;

impl Plugin for EssPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Styles>();

        // TODO: may be desabled with feature
        app.insert_resource(Defaults::default());
        app.add_systems(Startup, crate::ess::defaults::setup_defaults);

        app.init_asset::<StyleSheet>();
        let extractor = app
            .world
            .get_resource_or_insert_with(PropertyExtractor::default)
            .clone();
        let validator = app
            .world
            .get_resource_or_insert_with(PropertyTransformer::default)
            .clone();
        app.register_asset_loader(EssLoader {
            validator,
            extractor,
        });
        app.add_systems(Update, process_styles_system);
        app.add_plugins(property::PropertyPlugin);
        app.add_plugins(bevy_stylebox::StyleboxPlugin);

        // app.register_property::<impls::BackgroundColorProperty>();
        // app.register_property::<impls::ScaleProperty>();
    }
}

/// Possible errors that can be produced by [`EssAssetLoaderError`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum EssAssetLoaderError {
    /// EML parse error
    #[error("Could not parse ess: {0}")]
    ParseError(#[from] Error),
}

#[derive(Default)]
struct EssLoader {
    validator: PropertyTransformer,
    extractor: PropertyExtractor,
}

impl AssetLoader for EssLoader {
    type Settings = ();
    type Error = EssAssetLoaderError;
    type Asset = StyleSheet;

    fn extensions(&self) -> &[&str] {
        &["css", "ess"]
    }

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a Self::Settings,
        _: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut source = String::new();
            reader.read_to_string(&mut source).await.unwrap();
            let parser = StyleSheetParser::new(self.validator.clone(), self.extractor.clone());
            let rules = parser.parse(source.as_str());
            let mut stylesheet = StyleSheet::default();
            for rule in rules {
                stylesheet.add_rule(rule)
            }
            // load_context.set_default_asset(LoadedAsset::new(stylesheet));
            Ok(stylesheet)
        })
    }
}

#[derive(Default, TypeUuid, TypePath, Asset)]
#[uuid = "93767098-caca-4f2b-b1d3-cdc91919be75"]
pub struct StyleSheet {
    weight: usize,
    rules: Vec<StyleRule>,
}

unsafe impl Send for StyleSheet {}
unsafe impl Sync for StyleSheet {}

pub struct LoadCommand {
    path: String,
}

pub struct ParseCommand {
    source: String,
    default: bool,
}

impl Command for ParseCommand {
    fn apply(self, world: &mut bevy::prelude::World) {
        let world = world.cell();
        let extractor = world.resource::<PropertyExtractor>().clone();
        let validator = world.resource::<PropertyTransformer>().clone();
        let parser = StyleSheetParser::new(validator, extractor);
        let rules = parser.parse(&self.source);
        let stylesheet = StyleSheet::new(rules);
        let mut styles = world.resource_mut::<Styles>();
        let mut assets = world.resource_mut::<Assets<StyleSheet>>();
        let handle = assets.add(stylesheet);
        if self.default {
            world.resource_mut::<Defaults>().style_sheet = handle.clone();
        }
        styles.insert(handle);
    }
}

pub struct AddCommand {
    rules: SmallVec<[StyleRule; 8]>,
    default: bool,
}

impl Command for AddCommand {
    fn apply(self, world: &mut bevy::prelude::World) {
        let world = world.cell();
        let stylesheet = StyleSheet::new(self.rules);
        let mut styles = world.resource_mut::<Styles>();
        let mut assets = world.resource_mut::<Assets<StyleSheet>>();
        let handle = assets.add(stylesheet);
        if self.default {
            world.resource_mut::<Defaults>().style_sheet = handle.clone();
        }
        styles.insert(handle);
    }
}

impl Command for LoadCommand {
    fn apply(self, world: &mut bevy::prelude::World) {
        let world = world.cell();
        let mut styles = world.resource_mut::<Styles>();
        let handle = world.resource::<AssetServer>().load(&self.path);
        styles.insert(handle);
    }
}

impl StyleSheet {
    pub fn new<T: IntoIterator<Item = StyleRule>>(rules: T) -> StyleSheet {
        let mut sheet = Self::default();
        for rule in rules.into_iter() {
            sheet.add_rule(rule);
        }
        sheet
    }
    pub fn load(path: &str) -> LoadCommand {
        LoadCommand {
            path: path.to_string(),
        }
    }
    pub fn parse(source: &str) -> ParseCommand {
        ParseCommand {
            source: source.to_string(),
            default: false,
        }
    }
    pub fn parse_default(source: &str) -> ParseCommand {
        ParseCommand {
            source: source.to_string(),
            default: true,
        }
    }
    pub fn add(rules: SmallVec<[StyleRule; 8]>) -> AddCommand {
        AddCommand {
            rules,
            default: false,
        }
    }
    pub fn add_default(rules: SmallVec<[StyleRule; 8]>) -> AddCommand {
        AddCommand {
            rules,
            default: true,
        }
    }
    pub fn add_rule(&mut self, mut rule: StyleRule) {
        rule.selector.index = SelectorIndex::new(self.rules.len());
        self.rules.push(rule);
    }

    pub(crate) fn extra_weight(&self) -> usize {
        self.weight
    }

    pub(crate) fn set_extra_weight(&mut self, weight: usize) {
        self.weight = weight;
        self.rules
            .iter_mut()
            .for_each(|r| r.selector.weight.1 = weight as i32);
    }
}

impl Deref for StyleSheet {
    type Target = Vec<StyleRule>;

    fn deref(&self) -> &Self::Target {
        &self.rules
    }
}

#[derive(Debug)]
pub struct StyleRule {
    pub selector: Selector,
    // pub properties: HashMap<Tag, StyleProperty>,
    pub properties: HashMap<Tag, PropertyValue>,
}

#[derive(Default, Resource)]
pub struct Styles {
    last_id: usize,
    map: HashMap<Handle<StyleSheet>, usize>,
}

impl Styles {
    pub fn insert(&mut self, handle: Handle<StyleSheet>) -> usize {
        let default = self.last_id + 1;
        let id = *self.map.entry(handle).or_insert(default);
        if id > self.last_id {
            self.last_id = id;
        }
        id
    }

    pub fn iter(&self) -> Keys<Handle<StyleSheet>, usize> {
        self.map.keys()
    }

    pub fn weight(&self, handle: &Handle<StyleSheet>) -> usize {
        *self.map.get(handle).unwrap_or(&0)
    }
}

fn process_styles_system(
    asset_server: Res<AssetServer>,
    mut styles: ResMut<Styles>,
    mut assets: ResMut<Assets<StyleSheet>>,
    mut events: EventReader<AssetEvent<StyleSheet>>,
    mut elements: Elements,
    defaults: Res<Defaults>,
) {
    let mut styles_changed = false;
    for event in events.read() {
        styles_changed = true;
        match event {
            AssetEvent::Removed { id: _ } => styles_changed = true,
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => {
                if let Some(handle) = asset_server.get_id_handle(*id) {
                    if handle == defaults.style_sheet {
                        if assets.get(*id).unwrap().extra_weight() != 0 {
                            assets.get_mut(*id).unwrap().set_extra_weight(0);
                        }
                    } else {
                        let handle = asset_server.get_id_handle(*id).unwrap();
                        let weight = styles.insert(handle);
                        if assets.get(*id).unwrap().extra_weight() != weight {
                            assets.get_mut(*id).unwrap().set_extra_weight(weight);
                        }
                    }
                }
            }
        }
    }
    if styles_changed {
        elements.invalidate_all();
    }
}
