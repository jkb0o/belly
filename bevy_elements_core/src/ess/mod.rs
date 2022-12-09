mod parser;
mod selector;

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    ecs::system::Command,
    prelude::*,
    reflect::TypeUuid,
    utils::{hashbrown::hash_map::Keys, HashMap},
};
use itertools::Itertools;
pub use selector::*;
use smallvec::SmallVec;
use tagstr::Tag;

use crate::{
    input::invalidate_elements, property::StyleProperty, Defaults, Element, PropertyExtractor,
    PropertyTransformer, Variant,
};

pub use self::parser::StyleSheetParser;
use std::ops::Deref;

#[derive(Default)]
pub struct EssPlugin;

impl Plugin for EssPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Styles>();
        app.add_asset::<StyleSheet>();
        let extractor = app
            .world
            .get_resource_or_insert_with(PropertyExtractor::default)
            .clone();
        let validator = app
            .world
            .get_resource_or_insert_with(PropertyTransformer::default)
            .clone();
        app.add_asset_loader(EssLoader {
            validator,
            extractor,
        });
        app.add_system(process_styles_system);
    }
}

#[derive(Default)]
struct EssLoader {
    validator: PropertyTransformer,
    extractor: PropertyExtractor,
}

impl AssetLoader for EssLoader {
    fn extensions(&self) -> &[&str] {
        &["css", "ess"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let source = std::str::from_utf8(bytes)?;
            let parser = StyleSheetParser::new(self.validator.clone(), self.extractor.clone());
            let rules = parser.parse(source);
            let mut stylesheet = StyleSheet::default();
            for rule in rules {
                stylesheet.add_rule(rule)
            }
            load_context.set_default_asset(LoadedAsset::new(stylesheet));
            Ok(())
        })
    }
}

#[derive(Default, TypeUuid)]
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
    fn write(self, world: &mut bevy::prelude::World) {
        info!("wriging ParseCommand");
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
    fn write(self, world: &mut bevy::prelude::World) {
        info!("wriging AddCommand");
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
    fn write(self, world: &mut bevy::prelude::World) {
        info!("wriging LoadCommand");
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
    pub properties: HashMap<Tag, Variant>,
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
    mut styles: ResMut<Styles>,
    mut assets: ResMut<Assets<StyleSheet>>,
    mut events: EventReader<AssetEvent<StyleSheet>>,
    defaults: Res<Defaults>,
    roots: Query<Entity, (With<Element>, Without<Parent>)>,
    mut elements: Query<(Entity, &mut Element)>,
    children: Query<&Children>,
) {
    let mut styles_changed = false;
    for event in events.iter() {
        styles_changed = true;
        match event {
            AssetEvent::Removed { handle: _ } => styles_changed = true,
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                if handle == &defaults.style_sheet {
                    if assets.get(handle).unwrap().extra_weight() != 0 {
                        assets.get_mut(handle).unwrap().set_extra_weight(0);
                    }
                } else {
                    let weight = styles.insert(handle.clone());
                    if assets.get(handle).unwrap().extra_weight() != weight {
                        assets.get_mut(handle).unwrap().set_extra_weight(weight);
                    }
                }
            }
        }
    }
    if styles_changed {
        invalidate_elements(&roots, &mut elements, &children);
    }
}
