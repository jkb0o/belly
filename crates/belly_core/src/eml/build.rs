use std::{
    mem,
    sync::{Arc, RwLock},
};

use bevy::{
    asset::Asset,
    ecs::system::{Command, CommandQueue, EntityCommands},
    prelude::*,
    ui::FocusPolicy,
    utils::{HashMap, HashSet},
};
use itertools::Itertools;
use tagstr::{tag, Tag};

use crate::{
    element::Element,
    ess::{PropertyExtractor, PropertyTransformer, StyleRule, StyleSheetParser},
    relations::{ConnectionBuilder, Signal},
    tags,
};

use super::{Params, StyleParams, Variant};

pub struct BuildPlugin;
impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RequestReadyEvent>();
        app.add_event::<ReadyEvent>();
        app.add_system_to_stage(CoreStage::PostUpdate, emit_ready_signal);
        app.init_resource::<Slots>();
    }
}

#[derive(Resource, Clone, Default, Deref)]
pub struct WidgetRegistry(Arc<RwLock<HashMap<Tag, WidgetBuilder>>>);

impl WidgetRegistry {
    pub fn default_styles(&self, parser: &StyleSheetParser) -> Vec<StyleRule> {
        self.0
            .read()
            .unwrap()
            .values()
            .map(|b| b.default_styles())
            .flat_map(|s| parser.parse(s))
            .collect()
    }
}
impl WidgetRegistry {
    pub fn get<T: Into<Tag>>(&self, name: T) -> Option<WidgetBuilder> {
        self.0.read().unwrap().get(&name.into()).copied()
    }

    pub fn has<T: Into<Tag>>(&self, name: T) -> bool {
        self.0.read().unwrap().contains_key(&name.into())
    }
}

pub trait RegisterWidget {
    fn register_widget<T: Widget + Sync + Send + 'static>(&mut self) -> &mut Self;
}

impl RegisterWidget for App {
    fn register_widget<T: Widget + Sync + Send + 'static>(&mut self) -> &mut Self {
        let registry = self
            .world
            .get_resource_or_insert_with(WidgetRegistry::default)
            .clone();
        let widget = T::instance();
        let name = Widget::name(widget);

        registry.write().unwrap().insert(name, widget.as_builder());
        self
    }
}

/// Data collect by `eml!` macro ot `eml` asset and passed to
/// [`Widget::build_for_world`] for all heavy world
pub struct WidgetData {
    /// Widget entity (generated or provided by user)
    pub entity: Entity,
    /// Children already processed by [`Widget::build`]
    pub children: Vec<Entity>,
    /// Attributes defined within the tag
    pub params: Params,
}

impl WidgetData {
    pub fn new(entity: Entity) -> WidgetData {
        WidgetData {
            entity,
            children: vec![],
            params: Params::default(),
        }
    }
}

/// Context passed to widget builder func.
pub struct WidgetContext<'w, 's> {
    data: WidgetData,
    commands: Commands<'w, 's>,
    asset_server: AssetServer,
    extractor: PropertyExtractor,
    transformer: PropertyTransformer,
}

impl<'w, 's> WidgetContext<'w, 's> {
    pub fn this<'a>(&'a mut self) -> EntityCommands<'w, 's, 'a> {
        self.commands.entity(self.data.entity)
    }
    pub fn load<T: Asset>(&self, path: &str) -> Handle<T> {
        self.asset_server.load(path)
    }

    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.commands
    }

    pub fn empty(&mut self) -> Entity {
        self.commands.spawn_empty().id()
    }

    pub fn add<C: Command>(&mut self, command: C) {
        self.commands.add(command)
    }

    pub fn insert<'a>(&'a mut self, bundle: impl Bundle) -> EntityCommands<'w, 's, 'a> {
        let mut commands = self.commands.entity(self.data.entity);
        commands.insert(bundle);
        commands
    }

    pub fn render(&mut self, elements: Eml) {
        self.commands.add(elements.with_entity(self.data.entity));
    }

    pub fn entity(&self) -> Entity {
        self.data.entity
    }

    pub fn content(&mut self) -> Vec<Entity> {
        mem::take(&mut self.data.children)
    }

    pub fn param(&mut self, key: Tag) -> Option<Variant> {
        self.data.params.drop_variant(key)
    }

    pub fn params(&mut self) -> Params {
        mem::take(&mut self.data.params)
    }

    pub fn id(&mut self) -> Option<Tag> {
        self.data.params.id()
    }

    pub fn classes(&mut self) -> HashSet<Tag> {
        self.data.params.classes()
    }

    pub fn styles(&mut self) -> StyleParams {
        self.data.params.styles()
    }

    pub fn apply_commands(&mut self) {
        if let Some(attr_commands) = self.data.params.commands(tags::with()) {
            attr_commands(&mut self.commands.entity(self.entity()));
        }
    }

    pub fn update_element<F: FnOnce(&mut Element) + Send + Sync + 'static>(&mut self, update: F) {
        let entity = self.entity();
        self.commands.add(move |world: &mut World| {
            if let Some(mut element) = world.entity_mut(entity).get_mut::<Element>() {
                update(&mut element);
            } else {
                let mut element = Element::default();
                update(&mut element);
                world.entity_mut(entity).insert(element);
            }
        });
    }
}

/// A tuple of component types. It us used for instantiating
/// set of components and bypassing them `&mut Components` into [`BuildWidgetFunc`]:
/// `invoke_build_widget(&mut ComponentA, &mut ComponentB)
pub trait Components {
    fn instantiate(world: &mut World, params: &mut Params) -> Self;
    fn write(self, commands: EntityCommands);
}

/// The fuctions that can act like widget builders:
/// ```rust
/// # use bevy::prelude::*;
/// # use belly_core::eml::WidgetContext;
/// #[derive(Component, Default)]
/// struct ComponentA {
///     field: f32
/// }
/// #[derive(Component, Default)]
/// struct ComponentB {
///     field: String
/// }
/// fn my_widget(ctx: &mut WidgetContext, a: &mut ComponentA, b: &mut ComponentB) {
///     a.field = 1.0;
///     b.field = "1.0".into();
/// }
pub trait BuildWidgetFunc<Params: Components>: 'static {
    fn invoke_build_widget(&self, ctx: &mut WidgetContext, params: &mut Params);
}

macro_rules! impl_components {
    (@imp $($ident:ident,)*) => {
        impl <$($ident: Component + FromWorldAndParams,)*> Components for ($($ident,)*) {
            #[allow(unused_variables)]
            fn instantiate(world: &mut World, params: &mut Params) -> Self {
                ($($ident::from_world_and_params(world, params),)* )
            }
            fn write(self, mut commands: EntityCommands) {
                commands.insert(self);
            }
        }
        impl<Func: Fn(&mut WidgetContext, $(&mut $ident,)*) + 'static, $($ident: Component + FromWorldAndParams,)*> BuildWidgetFunc<($($ident,)*)> for Func {
            fn invoke_build_widget(&self, ctx: &mut WidgetContext, params: &mut ($($ident,)*)) {
                #[allow(non_snake_case)]
                let ($($ident,)*) = params;
                self(ctx, $($ident,)*);
            }
        }
    };
    ($ident:ident, $($rest:ident,)+) => {
        impl_components! { @imp $ident, $($rest,)+ }
        impl_components! { $($rest,)+ }
    };
    ($ident:ident,) => {
        impl_components! { @imp $ident, }
        impl_components! { @imp }
    };
}
impl_components! { A, B, C, D, E, F, G, H, }

/// Only the way I know to get the ref to the unit struct
pub trait Singleton: 'static {
    fn instance() -> &'static Self;
}

/// Instantiate components from world and params
pub trait FromWorldAndParams {
    fn from_world_and_params(world: &mut World, params: &mut Params) -> Self;
}

impl<T: Default> FromWorldAndParams for T {
    fn from_world_and_params(_world: &mut World, _params: &mut Params) -> Self {
        Self::default()
    }
}

pub trait FromWorldAndParam {
    fn from_world_and_param(world: &mut World, param: Variant) -> Self;
}

impl<T: Default> FromWorldAndParam for T {
    fn from_world_and_param(_world: &mut World, _param: Variant) -> Self {
        Self::default()
    }
}

/// Unified Widget API
pub trait Widget {
    /// Additional components inserted into the entity
    type Components: Components;
    type BuildComponents: Components;
    type OtherComponents: Components;
    /// Generated by `#[widget]` macro predefined bindigs from properties
    type BindingsFrom: Singleton;
    /// Generated by `#[widget]` macro predefined bindigs to properties
    type BindingsTo: Singleton;
    /// Generated by `#[widget]` macro predefined connections
    type Signals: Singleton;

    // TODO: implement Query protocol for widgets
    // /// Generated by `#[widget]` macro [`ReadOnlyWorldQuery`] implementation
    // type ReadQuery: ReadOnlyWorldQuery + 'static;
    // /// Generated by `#[widget]` macro [`WorldQuery`] implementation
    // type WriteQuery: WorldQuery + 'static;

    /// Obitain the reference to widget that is unit struct.
    fn instance() -> &'static Self;

    /// The widget name (it is the tag name)
    fn name(&self) -> Tag;

    /// Style alias for matchining extendent widgets. For example `<progressbar>`
    /// widgets extends `<range>` widget. It has name name `tag!("progressbar")`
    /// and alias `Some(tag!("range")). Any `range` style rule selectors will
    /// also affect the `progressbar`.
    fn alias(&self) -> Option<Tag> {
        None
    }

    /// This function indirectly implemented by user. It should populate the tree,
    /// add extra components, load assets, build the widget, you know.
    fn build_widget(&self, ctx: &mut WidgetContext, components: &mut Self::BuildComponents);

    /// Create instance of [`Self::Components`] based on provided widget attributes.
    /// This method is generated by `#[widget]` macro.
    fn instantiate_components(&self, world: &mut World, params: &mut Params) -> Self::Components;

    fn split_components(
        &self,
        components: Self::Components,
    ) -> (Self::BuildComponents, Self::OtherComponents);

    // TODO: implement attribute-based binds
    // fn bind_components(&self);

    /// Obitain access to binding from properties
    fn bind_from(&self) -> &Self::BindingsFrom {
        Self::BindingsFrom::instance()
    }
    /// Obitain access to bindings to components
    fn bind_to(&self) -> &Self::BindingsTo {
        Self::BindingsTo::instance()
    }

    /// Obitain access to signals
    // #[doc = include_str!("../../hello.md")]
    fn on(&self) -> &Self::Signals {
        Self::Signals::instance()
    }

    fn build(&self, world: &mut World, mut data: WidgetData) {
        let components = self.instantiate_components(world, &mut data.params);
        let mut queue = CommandQueue::default();
        let commands = Commands::new(&mut queue, world);
        let asset_server = world.resource::<AssetServer>().clone();
        let transformer = world.resource::<PropertyTransformer>().clone();
        let extractor = world.resource::<PropertyExtractor>().clone();
        let mut ctx = WidgetContext {
            data,
            commands,
            asset_server,
            transformer,
            extractor,
        };

        // TODO: implement attribute-based binds
        // self.bind_components(&mut ctx, &components);
        let (mut build_components, other_components) = self.split_components(components);
        self.build_widget(&mut ctx, &mut build_components);
        build_components.write(ctx.this());
        other_components.write(ctx.this());

        // post process
        ctx.apply_commands();
        let focus_policy = match ctx.param(tag!("interactable")) {
            Some(Variant::Bool(true)) => Some(FocusPolicy::Block),
            Some(Variant::String(s)) if &s == "block" => Some(FocusPolicy::Block),
            Some(Variant::String(s)) if &s == "pass" => Some(FocusPolicy::Pass),
            _ => None,
        };
        if let Some(policy) = focus_policy {
            ctx.insert(policy);
            ctx.insert(Interaction::default());
        }
        let names = vec![self.name()].into();
        let aliases = if let Some(alias) = self.alias() {
            vec![alias].into()
        } else {
            vec![].into()
        };
        let id = ctx.id();
        let classes = ctx.classes();
        let styles = ctx.styles().transform(|tag, variant| {
            if ctx.extractor.is_compound_property(tag) {
                match ctx.extractor.extract(tag, variant) {
                    Ok(mut props) => props.drain().collect(),
                    Err(e) => {
                        error!("Ignoring property {}: {}", tag, e);
                        vec![]
                    }
                }
            } else {
                match ctx.transformer.transform(tag, variant) {
                    Ok(variant) => vec![(tag, variant)],
                    Err(e) => {
                        error!("Ignoring property {}: {}", tag, e);
                        vec![]
                    }
                }
            }
        });
        let entity = ctx.entity();
        ctx.commands.add(move |world: &mut World| {
            world
                .resource_mut::<Events<RequestReadyEvent>>()
                .send(RequestReadyEvent(entity));
        });
        ctx.update_element(move |element| {
            element.names = names;
            element.aliases = aliases;
            element.id = id;
            element.classes.extend(classes);
            element.styles.extend(styles);
        });

        queue.apply(world)
    }
    fn default_styles(&self) -> &str {
        ""
    }
    fn as_builder(&'static self) -> WidgetBuilder
    where
        Self: Sized + Sync + Send + 'static,
    {
        WidgetBuilder(self)
    }
}

#[derive(Clone, Copy)]
pub struct WidgetBuilder(&'static dyn WidgetUntyped);
impl WidgetBuilder {
    pub fn name(&self) -> Tag {
        self.0.name()
    }
    pub fn build(&self, world: &mut World, data: WidgetData) {
        self.0.build(world, data)
    }
    pub fn default_styles(&self) -> &str {
        self.0.default_styles()
    }
}

pub trait WidgetUntyped: Send + Sync {
    /// The widget name (it is the tag name)
    fn name(&self) -> Tag;

    fn build(&self, world: &mut World, data: WidgetData);

    fn default_styles(&self) -> &str;
}

impl<T: Widget + Send + Sync> WidgetUntyped for T {
    fn name(&self) -> Tag {
        self.name()
    }
    fn build(&self, world: &mut World, data: WidgetData) {
        self.build(world, data)
    }
    fn default_styles(&self) -> &str {
        self.default_styles()
    }
}

pub struct DefaultBindingsFrom;
pub struct DefaultBindingsTo;
pub struct DefaultSignals;

#[derive(PartialEq, Eq, Hash)]
pub struct RequestReadyEvent(pub(crate) Entity);
pub struct ReadyEvent([Entity; 1]);

impl Signal for ReadyEvent {
    fn sources(&self) -> &[Entity] {
        &self.0
    }
}

fn emit_ready_signal(
    mut requests: EventReader<RequestReadyEvent>,
    mut writer: EventWriter<ReadyEvent>,
) {
    for req in requests.iter().unique() {
        writer.send(ReadyEvent([req.0]))
    }
}

impl DefaultSignals {
    pub fn ready<C: Component, F: Fn(&mut ConnectionBuilder<C, ReadyEvent>)>(
        &self,
        world: &mut World,
        source: Entity,
        connect: F,
    ) {
        let mut builder = ConnectionBuilder::<C, ReadyEvent>::default();
        connect(&mut builder);
        if let Some(target) = builder.build() {
            target.filter(|_| true).from(source).write(world)
        } else {
            error!("Unable to create connection to `ready`");
        }
    }
}

pub struct Eml {
    pub builder: Box<dyn FnOnce(&mut World, Entity) + Sync + Send>,
}

impl Eml {
    pub fn new<T>(builder: T) -> Self
    where
        T: FnOnce(&mut World, Entity) + Sync + Send + 'static,
    {
        Eml {
            builder: Box::new(builder),
        }
    }

    pub fn with_entity(self, entity: Entity) -> impl FnOnce(&mut World) {
        move |world: &mut World| {
            (self.builder)(world, entity);
        }
    }
}

impl Command for Eml {
    fn write(self, world: &mut World) {
        let entity = world.spawn_empty().id();
        self.with_entity(entity)(world);
    }
}

#[derive(Resource, Default, Clone)]
pub struct Slots(Arc<RwLock<HashMap<Tag, Vec<Entity>>>>);

impl Slots {
    pub fn insert(&self, tag: Tag, entities: Vec<Entity>) {
        self.0.write().unwrap().insert(tag, entities);
    }

    pub fn remove(&self, tag: Tag) -> Option<Vec<Entity>> {
        self.0.write().unwrap().remove(&tag)
    }

    pub fn keys(&self) -> HashSet<Tag> {
        self.0.read().unwrap().keys().copied().collect()
    }
}
