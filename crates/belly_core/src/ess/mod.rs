mod defaults;
mod parser;
mod property;
mod selector;
mod stylebox;

pub use self::parser::StyleSheetParser;
use crate::{element::Elements, eml::Variant, ess::defaults::Defaults, ElementsError};
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    ecs::system::Command,
    prelude::*,
    reflect::TypeUuid,
    text::TextLayoutInfo,
    utils::{hashbrown::hash_map::Keys, HashMap},
};
pub use property::*;
pub use selector::*;
use smallvec::SmallVec;
use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};
use tagstr::Tag;

#[derive(Default)]
pub struct EssPlugin;

impl Plugin for EssPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Styles>();

        // TODO: may be desabled with feature
        app.insert_resource(Defaults::default());
        app.add_startup_system(crate::ess::defaults::setup_defaults);

        app.add_asset::<StyleSheet>();
        app.add_system(fix_text_height);
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
        app.add_plugin(bevy_stylebox::StyleboxPlugin);
        app.add_plugin(stylebox::StyleboxPropertyPlugin);

        app.register_property::<impls::DisplayProperty>();
        app.register_property::<impls::PositionTypeProperty>();
        app.register_property::<impls::DirectionProperty>();
        app.register_property::<impls::FlexDirectionProperty>();
        app.register_property::<impls::FlexWrapProperty>();
        app.register_property::<impls::AlignItemsProperty>();
        app.register_property::<impls::AlignSelfProperty>();
        app.register_property::<impls::AlignContentProperty>();
        app.register_property::<impls::JustifyContentProperty>();
        app.register_property::<impls::OverflowProperty>();

        app.register_property::<impls::WidthProperty>();
        app.register_property::<impls::HeightProperty>();
        app.register_property::<impls::MinWidthProperty>();
        app.register_property::<impls::MinHeightProperty>();
        app.register_property::<impls::MaxWidthProperty>();
        app.register_property::<impls::MaxHeightProperty>();
        app.register_property::<impls::FlexBasisProperty>();
        app.register_property::<impls::FlexGrowProperty>();
        app.register_property::<impls::FlexShrinkProperty>();
        app.register_property::<impls::AspectRatioProperty>();

        app.register_compound_property::<impls::PositionProperty>();
        app.register_property::<impls::LeftProperty>();
        app.register_property::<impls::RightProperty>();
        app.register_property::<impls::TopProperty>();
        app.register_property::<impls::BottomProperty>();

        app.register_compound_property::<impls::PaddingProperty>();
        app.register_property::<impls::PaddingLeftProperty>();
        app.register_property::<impls::PaddingRightProperty>();
        app.register_property::<impls::PaddingTopProperty>();
        app.register_property::<impls::PaddingBottomProperty>();

        app.register_compound_property::<impls::MarginProperty>();
        app.register_property::<impls::MarginLeftProperty>();
        app.register_property::<impls::MarginRightProperty>();
        app.register_property::<impls::MarginTopProperty>();
        app.register_property::<impls::MarginBottomProperty>();

        app.register_compound_property::<impls::BorderProperty>();
        app.register_property::<impls::BorderLeftProperty>();
        app.register_property::<impls::BorderRightProperty>();
        app.register_property::<impls::BorderTopProperty>();
        app.register_property::<impls::BorderBottomProperty>();

        app.register_property::<impls::FontColorProperty>();
        app.register_property::<impls::FontProperty>();
        app.register_property::<impls::FontSizeProperty>();
        app.register_property::<impls::VerticalAlignProperty>();
        app.register_property::<impls::HorizontalAlignProperty>();
        app.register_property::<impls::TextContentProperty>();

        app.register_property::<impls::BackgroundColorProperty>();
        app.register_property::<impls::ScaleProperty>();
    }
}

pub(crate) type TransformProperty = fn(Variant) -> Result<PropertyValue, ElementsError>;
#[derive(Default, Clone, Resource)]
pub struct PropertyTransformer(Arc<RwLock<HashMap<Tag, TransformProperty>>>);
unsafe impl Send for PropertyTransformer {}
unsafe impl Sync for PropertyTransformer {}
impl PropertyTransformer {
    #[cfg(test)]
    pub(crate) fn new(rules: HashMap<Tag, TransformProperty>) -> PropertyTransformer {
        PropertyTransformer(Arc::new(RwLock::new(rules)))
    }
    pub(crate) fn transform(
        &self,
        name: Tag,
        value: Variant,
    ) -> Result<PropertyValue, ElementsError> {
        self.0
            .read()
            .unwrap()
            .get(&name)
            .ok_or(ElementsError::UnsupportedProperty(name.to_string()))
            .and_then(|transform| transform(value))
    }
}

pub(crate) type ExtractProperty = fn(Variant) -> Result<HashMap<Tag, PropertyValue>, ElementsError>;
#[derive(Default, Clone, Resource)]
pub struct PropertyExtractor(Arc<RwLock<HashMap<Tag, ExtractProperty>>>);
unsafe impl Send for PropertyExtractor {}
unsafe impl Sync for PropertyExtractor {}
impl PropertyExtractor {
    #[cfg(test)]
    pub(crate) fn new(rules: HashMap<Tag, ExtractProperty>) -> PropertyExtractor {
        PropertyExtractor(Arc::new(RwLock::new(rules)))
    }
    pub(crate) fn is_compound_property(&self, name: Tag) -> bool {
        self.0.read().unwrap().contains_key(&name)
    }

    pub(crate) fn extract(
        &self,
        name: Tag,
        value: Variant,
    ) -> Result<HashMap<Tag, PropertyValue>, ElementsError> {
        self.0
            .read()
            .unwrap()
            .get(&name)
            .ok_or(ElementsError::UnsupportedProperty(name.to_string()))
            .and_then(|extractor| extractor(value))
    }
}

pub trait RegisterProperty {
    fn register_property<T: Property + 'static>(&mut self) -> &mut Self;
    fn register_compound_property<T: CompoundProperty + 'static>(&mut self) -> &mut Self;
}

impl RegisterProperty for bevy::prelude::App {
    fn register_property<T: Property + 'static>(&mut self) -> &mut Self {
        self.world
            .get_resource_or_insert_with(PropertyTransformer::default)
            .0
            .write()
            .unwrap()
            .entry(T::name())
            .and_modify(|_| panic!("Property `{}` already registered.", T::name()))
            .or_insert(T::transform);
        self.add_system(T::apply_defaults /* .label(EcssSystem::Apply) */);
        self
    }

    fn register_compound_property<T: CompoundProperty + 'static>(&mut self) -> &mut Self {
        self.world
            .get_resource_or_insert_with(PropertyExtractor::default)
            .0
            .write()
            .unwrap()
            .entry(T::name())
            .and_modify(|_| panic!("CompoundProperty `{}` already registered", T::name()))
            .insert(T::extract);
        self
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
    mut styles: ResMut<Styles>,
    mut assets: ResMut<Assets<StyleSheet>>,
    mut events: EventReader<AssetEvent<StyleSheet>>,
    mut elements: Elements,
    defaults: Res<Defaults>,
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
        elements.invalidate_all();
    }
}

pub fn fix_text_height(
    mut texts: Query<(&Text, &mut Style), Or<(Changed<Text>, Changed<TextLayoutInfo>)>>,
) {
    for (text, mut style) in texts.iter_mut() {
        if text.sections.len() > 0 {
            style.size.height = Val::Px(text.sections[0].style.font_size);
        }
    }
}
