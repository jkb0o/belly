mod parser;
mod selector;

use bevy::{asset::{AssetLoader, LoadedAsset}, prelude::{Handle, AssetServer, Res, Assets, ResMut, EventReader, AssetEvent, Query, Plugin, AddAsset}, utils::{HashSet, HashMap}, reflect::TypeUuid, ecs::system::Command};
pub use selector::*;
use tagstr::{AsTag, Tag};

use crate::{PropertyValidator, PropertyExtractor, element::Element, property::PropertyValues};

use self::parser::StyleSheetParser;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct EssPlugin;

impl Plugin for EssPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Styles>();
        app.add_asset::<Stylesheet>();
        let extractor = app.world.get_resource_or_insert_with(PropertyExtractor::default).clone();
        let validator = app.world.get_resource_or_insert_with(PropertyValidator::default).clone();
        app.add_asset_loader(EssLoader { validator, extractor });
    }
}

#[derive(Default)]
struct EssLoader {
    validator: PropertyValidator,
    extractor: PropertyExtractor
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

            let rules = StyleSheetParser::parse(source, self.validator.clone(), self.extractor.clone());
            let mut stylesheet = Stylesheet::default();
            for rule in rules {
                stylesheet.add_rule(rule)
            };
            load_context.set_default_asset(LoadedAsset::new(stylesheet));
            Ok(())
        })
    }
}

#[derive(Default, TypeUuid)]
#[uuid = "93767098-caca-4f2b-b1d3-cdc91919be75"]
pub struct Stylesheet {
    rules: Vec<StyleRule>
}

unsafe impl Send for Stylesheet { }
unsafe impl Sync for Stylesheet { }

pub struct LoadCommand {
    path: String
}

pub struct ParseCommand {
    source: String
}

impl Command for ParseCommand {
    fn write(self, world: &mut bevy::prelude::World) {
        let world = world.cell();
        let extractor = world.resource::<PropertyExtractor>().clone();
        let validator = world.resource::<PropertyValidator>().clone();
        let rules = StyleSheetParser::parse(&self.source, validator, extractor);
        let stylesheet = Stylesheet::new(rules);
        let mut styles = world.resource_mut::<Styles>();
        let mut assets = world.resource_mut::<Assets<Stylesheet>>();
        let handle = assets.add(stylesheet);
        styles.insert(handle);
    }
}

impl Command for LoadCommand {
    fn write(self, world: &mut bevy::prelude::World) {
        let world = world.cell();
        let mut styles = world.resource_mut::<Styles>();
        let handle = world.resource::<AssetServer>().load(&self.path);
        styles.insert(handle);
    }
}

impl Stylesheet {
    pub fn new<T:IntoIterator<Item=StyleRule>>(rules: T) -> Stylesheet {
        let mut sheet = Self::default();
        for rule in rules.into_iter() {
            sheet.add_rule(rule);
        }
        sheet
    }
    pub fn load(path: &str) -> LoadCommand {
        LoadCommand { path: path.to_string() }
    }
    pub fn parse(source: &str) -> ParseCommand {
        ParseCommand { source: source.to_string() }
    }
    pub fn add_rule(&mut self, mut rule: StyleRule) {
        rule.selector.index = SelectorIndex::new(self.rules.len());
        self.rules.push(rule);
    }
}



impl Deref for Stylesheet {
    type Target = Vec<StyleRule>;

    fn deref(&self) -> &Self::Target {
        &self.rules
    }
}

pub struct StyleRule {
    pub selector: Selector,
    pub properties: HashMap<Tag, PropertyValues>
}

#[derive(Default)]
pub struct Styles(HashSet<Handle<Stylesheet>>);

impl Deref for Styles {
    type Target = HashSet<Handle<Stylesheet>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Styles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn test_iter(
    asset_server: Res<AssetServer>,
    styles: Res<Styles>,
    assets: Assets<Stylesheet>
) {
    let prop = "hello".as_tag();
    let rules: Vec<_> = styles
        .iter()
        .filter_map(|h| assets.get(h))
        .flat_map(|s| s.iter())
        .filter(|r| r.properties.contains_key(&prop))
        .collect();
}

fn test_add(
    asset_server: Res<AssetServer>,
    mut styles: ResMut<Styles>
) {
    styles.insert(asset_server.load("defaults.css"));
}

fn test_change(
    events: EventReader<AssetEvent<Stylesheet>>,
    mut elements: Query<&mut Element>
) {
    if events.len() == 0 {
        return;
    }
    for mut element in elements.iter_mut() {
        element.invalidate();
    }
}