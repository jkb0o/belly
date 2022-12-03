use std::{
    mem,
    sync::{Arc, RwLock},
};

use bevy::{
    asset::Asset,
    ecs::system::{Command, CommandQueue, EntityCommands},
    prelude::{App, AssetServer, Bundle, Commands, Component, Entity, Handle, Resource, World},
    ui::{FocusPolicy, Interaction},
    utils::{HashMap, HashSet},
};
use tagstr::*;

use crate::{attributes::Attributes, property::PropertyValues, tags, AttributeValue, Element};

pub trait Widget: Sized + Component + 'static {
    fn names() -> &'static [&'static str];

    fn default_styles() -> &'static str {
        ""
    }

    #[allow(unused_variables)]
    fn construct_component(world: &mut World) -> Option<Self> {
        None
    }

    #[allow(unused_variables)]
    fn bind_component(&mut self, ctx: &mut ElementContext) {}
}

pub trait WidgetBuilder: Widget {
    fn build(world: &mut World, data: ElementContextData) {
        let component = Self::construct_component(world);
        let mut queue = CommandQueue::default();
        let commands = Commands::new(&mut queue, world);
        let asset_server = world.resource::<AssetServer>().clone();
        let mut ctx = ElementContext {
            commands,
            data,
            asset_server,
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
    #[allow(unused_variables)]
    fn setup(&mut self, ctx: &mut ElementContext) {
        panic!("Not implemented")
    }
    #[allow(unused_variables)]
    fn construct(ctx: &mut ElementContext) {
        panic!("Not implemented")
    }

    fn post_process(ctx: &mut ElementContext) {
        let tag = Self::names().iter().next().unwrap().as_tag();
        ctx.apply_commands();
        let focus_policy = match ctx.param(tag!("interactable")) {
            Some(AttributeValue::Empty) => Some(FocusPolicy::Pass),
            Some(AttributeValue::String(s)) if &s == "block" => Some(FocusPolicy::Block),
            Some(AttributeValue::String(s)) if &s == "pass" => Some(FocusPolicy::Pass),
            _ => None,
        };
        if let Some(policy) = focus_policy {
            ctx.insert(policy);
            ctx.insert(Interaction::default());
        }
        let id = ctx.id();
        let classes = ctx.classes();
        let styles = ctx.styles();
        ctx.update_element(move |element| {
            element.name = Some(tag);
            element.id = id;
            element.classes.extend(classes);
            element.styles.extend(styles);
        });
    }

    fn as_builder() -> ElementBuilder {
        ElementBuilder {
            build_func: |world, ctx| Self::build(world, ctx),
            names_func: Self::names,
        }
    }
}

pub struct ElementContextData {
    pub entity: Entity,
    pub names: Names,
    pub children: Vec<Entity>,
    pub attributes: Attributes,
}

impl ElementContextData {
    pub fn new(entity: Entity) -> ElementContextData {
        ElementContextData {
            entity,
            names: || &[],
            children: vec![],
            attributes: Default::default(),
        }
    }
}

pub struct ElementContext<'w, 's> {
    data: ElementContextData,
    commands: Commands<'w, 's>,
    asset_server: AssetServer,
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

    pub fn param(&mut self, key: Tag) -> Option<AttributeValue> {
        self.data.attributes.drop_variant(key)
    }

    pub fn id(&mut self) -> Option<Tag> {
        self.data.attributes.id()
    }

    pub fn classes(&mut self) -> HashSet<Tag> {
        self.data.attributes.classes()
    }

    pub fn styles(&mut self) -> HashMap<Tag, PropertyValues> {
        self.data.attributes.styles()
    }

    pub fn apply_commands(&mut self) {
        if let Some(attr_commands) = self.data.attributes.commands(tags::with()) {
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

    fn release(self) -> ElementContextData {
        self.data
    }
}

type Names = fn() -> &'static [&'static str];

#[derive(Clone, Copy)]
pub struct ElementBuilder {
    build_func: fn(&mut World, ElementContextData),
    names_func: Names,
}

impl ElementBuilder {
    pub fn names(&self) -> impl Iterator<Item = Tag> {
        (self.names_func)().iter().map(|s| s.as_tag())
    }

    pub fn build(&self, world: &mut World, ctx: ElementContextData) {
        (self.build_func)(world, ctx);
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
}

pub trait RegisterWidgetExtension {
    fn register_widget<W: WidgetBuilder>(&mut self);
}

impl RegisterWidgetExtension for App {
    fn register_widget<W: WidgetBuilder>(&mut self) {
        let registry = self
            .world
            .get_resource_or_insert_with(ElementBuilderRegistry::default);
        for name in W::names().iter().map(|n| n.as_tag()) {
            registry.add_builder(name, W::as_builder());
        }
    }
}
