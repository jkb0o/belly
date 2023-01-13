use crate::{
    element::Element,
    eml::Params,
    eml::StyleParams,
    eml::Variant,
    ess::PropertyExtractor,
    ess::PropertyTransformer,
    ess::StyleRule,
    ess::StyleSheetParser,
    relations::Signal,
    relations::{connect::WithoutComponent, ConnectionTo},
    tags,
};
use bevy::{
    asset::Asset,
    ecs::system::{Command, CommandQueue, EntityCommands},
    prelude::*,
    ui::FocusPolicy,
    utils::{HashMap, HashSet},
};
use itertools::Itertools;
use std::{
    mem,
    sync::{Arc, RwLock},
};
use tagstr::*;

pub struct BuildPligin;
impl Plugin for BuildPligin {
    fn build(&self, app: &mut App) {
        app.add_event::<RequestReadyEvent>();
        app.add_event::<ReadyEvent>();
        app.init_resource::<Slots>();
        app.add_system_to_stage(CoreStage::PostUpdate, emit_ready_signal);
    }
}

pub trait FromWorldAndParam {
    fn from_world_and_param(world: &mut World, param: Variant) -> Self;
}

impl<T: TryFrom<Variant, Error = impl std::fmt::Display> + Default + 'static> FromWorldAndParam
    for T
{
    fn from_world_and_param(_world: &mut World, param: Variant) -> Self {
        if let Some(value) = param.try_get::<Self>() {
            value
        } else {
            Self::default()
        }
    }
}

pub fn entity_from_world_and_param(world: &mut World, param: Variant) -> Entity {
    if let Some(entity) = param.take::<Entity>() {
        entity
    } else {
        world.spawn_empty().id()
    }
}

pub trait Widget: Sized + Component + 'static {
    fn names() -> &'static [&'static str];
    fn aliases() -> &'static [&'static str] {
        &[]
    }

    #[allow(unused_variables)]
    fn construct_component(world: &mut World, params: &mut Params) -> Option<Self> {
        None
    }

    #[allow(unused_variables)]
    fn bind_component(&mut self, ctx: &mut ElementContext) {}
}

pub trait WidgetBuilder: Widget {
    #[allow(unused_variables)]
    fn setup(&mut self, ctx: &mut ElementContext) {
        panic!("Not implemented")
    }
    #[allow(unused_variables)]
    fn construct(ctx: &mut ElementContext) {
        panic!("Not implemented")
    }

    fn styles() -> &'static str {
        ""
    }

    fn build(world: &mut World, mut data: ElementContextData) {
        let component = Self::construct_component(world, &mut data.params);
        let mut queue = CommandQueue::default();
        let commands = Commands::new(&mut queue, world);
        let asset_server = world.resource::<AssetServer>().clone();
        let transformer = world.resource::<PropertyTransformer>().clone();
        let extractor = world.resource::<PropertyExtractor>().clone();
        let mut ctx = ElementContext {
            commands,
            data,
            asset_server,
            transformer,
            extractor,
        };
        if let Some(mut component) = component {
            component.bind_component(&mut ctx);
            component.setup(&mut ctx);
            ctx.insert(component);
        } else {
            Self::construct(&mut ctx);
        }
        Self::post_process(&mut ctx);
        queue.apply(world);
    }

    fn post_process(ctx: &mut ElementContext) {
        let names = Self::names().iter().map(|n| n.as_tag()).collect();
        let aliases = Self::aliases().iter().map(|n| n.as_tag()).collect();
        // println!("adding tag {}", names.ite
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
    }

    fn as_builder() -> ElementBuilder {
        ElementBuilder {
            build_func: |world, ctx| Self::build(world, ctx),
            styles_func: Self::styles,
            names_func: Self::names,
            aliases_func: Self::aliases,
        }
    }
}

pub struct ElementContextData {
    pub entity: Entity,
    pub names: Names,
    pub children: Vec<Entity>,
    pub params: Params,
}

impl ElementContextData {
    pub fn new(entity: Entity) -> ElementContextData {
        ElementContextData {
            entity,
            names: || &[],
            children: vec![],
            params: Default::default(),
        }
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

pub struct ElementContext<'w, 's> {
    data: ElementContextData,
    commands: Commands<'w, 's>,
    asset_server: AssetServer,
    extractor: PropertyExtractor,
    transformer: PropertyTransformer,
    // extractors
}

impl<'w, 's> ElementContext<'w, 's> {
    pub fn load<T: Asset>(&self, path: &str) -> Handle<T> {
        self.asset_server.load(path)
    }

    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.commands
    }

    pub fn empty(&mut self) -> Entity {
        self.commands.spawn_empty().id()
    }

    pub fn insert<'a>(&'a mut self, bundle: impl Bundle) -> EntityCommands<'w, 's, 'a> {
        let mut commands = self.commands.entity(self.data.entity);
        commands.insert(bundle);
        commands
    }

    pub fn render(&mut self, elements: ElementsBuilder) {
        self.commands.add(elements.with_entity(self.data.entity));
    }

    pub fn entity(&self) -> Entity {
        self.data.entity
    }

    pub fn content(&mut self) -> Vec<Entity> {
        mem::take(&mut self.data.children)
    }

    pub fn names(&self) -> impl Iterator<Item = Tag> {
        (self.data.names)().iter().map(|n| n.as_tag())
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

type Names = fn() -> &'static [&'static str];

#[derive(Clone, Copy)]
pub struct ElementBuilder {
    build_func: fn(&mut World, ElementContextData),
    styles_func: fn() -> &'static str,
    aliases_func: Names,
    names_func: Names,
}

impl ElementBuilder {
    pub fn names(&self) -> impl Iterator<Item = Tag> {
        (self.names_func)().iter().map(|s| s.as_tag())
    }

    pub fn aliases(&self) -> impl Iterator<Item = Tag> {
        (self.aliases_func)().iter().map(|s| s.as_tag())
    }

    pub fn build(&self, world: &mut World, ctx: ElementContextData) {
        (self.build_func)(world, ctx);
    }

    pub fn styles(&self) -> &'static str {
        (self.styles_func)()
    }
}

pub struct ElementsBuilder {
    pub builder: Box<dyn FnOnce(&mut World, Entity) + Sync + Send>,
}

impl ElementsBuilder {
    pub fn new<T>(builder: T) -> Self
    where
        T: FnOnce(&mut World, Entity) + Sync + Send + 'static,
    {
        ElementsBuilder {
            builder: Box::new(builder),
        }
    }

    pub fn with_entity(self, entity: Entity) -> impl FnOnce(&mut World) {
        move |world: &mut World| {
            (self.builder)(world, entity);
        }
    }
}

impl Command for ElementsBuilder {
    fn write(self, world: &mut World) {
        let entity = world.spawn_empty().id();
        self.with_entity(entity)(world);
    }
}

#[derive(Resource, Default, Clone)]
pub struct ElementBuilderRegistry(Arc<RwLock<HashMap<Tag, ElementBuilder>>>);

impl ElementBuilderRegistry {
    pub fn add_builder(&self, name: Tag, builder: ElementBuilder) {
        self.0.write().unwrap().insert(name, builder);
    }

    pub fn get_builder(&self, name: Tag) -> Option<ElementBuilder> {
        self.0.read().unwrap().get(&name).map(|b| *b)
    }

    pub fn has_builder(&self, name: Tag) -> bool {
        self.0.read().unwrap().contains_key(&name)
    }

    pub fn styles(&self, parser: StyleSheetParser) -> Vec<StyleRule> {
        self.0
            .read()
            .unwrap()
            .values()
            .map(|b| b.styles())
            .flat_map(|s| parser.parse(s))
            .collect()
    }
}

pub trait RegisterWidgetExtension {
    fn register_widget<W: WidgetBuilder>(&mut self) -> &mut Self;
}

impl RegisterWidgetExtension for App {
    fn register_widget<W: WidgetBuilder>(&mut self) -> &mut Self {
        let registry = self
            .world
            .get_resource_or_insert_with(ElementBuilderRegistry::default);
        for name in W::names().iter().map(|n| n.as_tag()) {
            registry.add_builder(name, W::as_builder());
        }
        self
    }
}

pub struct DefaultDescriptor;
impl DefaultDescriptor {
    pub fn get_instance() -> &'static DefaultDescriptor {
        &&DefaultDescriptor
    }

    pub fn ready<C: Component>(
        &self,
        world: &mut World,
        source: Entity,
        target: ConnectionTo<C, ReadyEvent>,
    ) {
        target.all().from(source).write(world)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct RequestReadyEvent(Entity);
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
