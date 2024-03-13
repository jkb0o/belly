use crate::{element::Elements, relations::RelationsSystems};
use bevy::{
    asset::Asset,
    ecs::{
        event::Event,
        query::{QueryItem, WorldQuery},
        system::{Command, EntityCommands},
    },
    prelude::*,
    utils::HashMap,
};
use std::{
    any::{type_name, Any, TypeId},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
};

pub type WorldEvent<E> = fn(&E) -> bool;
pub type EntityEvent<E> = fn(&E) -> EventSource;

pub enum EventFilter<E: Event> {
    World(WorldEvent<E>),
    Entity(EntityEvent<E>),
}

impl<E: Event> std::hash::Hash for EventFilter<E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Entity(f) => (f as *const EntityEvent<E>).hash(state),
            Self::World(f) => (f as *const WorldEvent<E>).hash(state),
        }
    }
}

impl<E: Event> PartialEq for EventFilter<E> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Entity(f0), Self::Entity(f1)) => {
                (f0 as *const EntityEvent<E>) == (f1 as *const EntityEvent<E>)
            }
            (Self::World(f0), Self::World(f1)) => {
                (f0 as *const WorldEvent<E>) == (f1 as *const WorldEvent<E>)
            }
            _ => false,
        }
    }
}

impl<E: Event> Eq for EventFilter<E> {}

impl<E: Event> EventFilter<E> {
    pub fn entity(filter: EntityEvent<E>) -> Self {
        Self::Entity(filter)
    }
    pub fn world(filter: WorldEvent<E>) -> Self {
        Self::World(filter)
    }

    pub fn func<F: 'static + Fn(&mut EventContext<E>)>(self, func: F) -> Connection<(), E> {
        Connection {
            target: None,
            source: None,
            handler: Handler(Box::new(move |ctx, _| func(ctx))),
            filter: self,
        }
    }
    pub fn handle<Q: WorldQuery, F: 'static + Fn(&mut EventContext<E>, &mut QueryItem<Q>)>(
        self,
        (_, target, handler): (PhantomData<Q>, Option<Entity>, F),
    ) -> Connection<Q, E> {
        Connection {
            target,
            source: None,
            handler: Handler(Box::new(handler)),
            filter: self,
        }
    }
}

pub enum EventSource<'a> {
    None,
    Single(Option<Entity>),
    Vec(usize, &'a Vec<Entity>),
    Iter(&'a mut dyn Iterator<Item = Entity>),
}

impl<'a> Iterator for EventSource<'a> {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::None => None,
            Self::Single(single) => mem::take(single),
            Self::Vec(idx, vec) => {
                let pos = *idx;
                *idx += 1;
                vec.get(pos).map(|e| *e)
            }
            Self::Iter(iter) => iter.next(),
        }
    }
}

impl<'a> EventSource<'a> {
    pub fn none() -> Self {
        Self::None
    }
    pub fn single(entity: Entity) -> Self {
        Self::Single(Some(entity))
    }

    pub fn vec(vec: &'a Vec<Entity>) -> Self {
        Self::Vec(0, vec)
    }
}

impl<'a> From<&'a Vec<Entity>> for EventSource<'a> {
    fn from(value: &'a Vec<Entity>) -> Self {
        EventSource::Vec(0, value)
    }
}

pub trait EntityIteratorExtension {
    fn entities<'a>(&'a mut self) -> EventSource<'a>;
}

impl<T: Iterator<Item = Entity>> EntityIteratorExtension for T {
    fn entities<'a>(&'a mut self) -> EventSource<'a> {
        EventSource::Iter(self)
    }
}
pub struct EventContext<'a, 'w, 's, E: Event + 'static> {
    pub(crate) source_event: &'a E,
    pub(crate) time_resource: &'a Time,
    pub(crate) asset_server: AssetServer,
    pub(crate) elements: &'a mut Elements<'w, 's>,
}

impl<'a, 'w, 's, E: Event> EventContext<'a, 'w, 's, E> {
    pub fn event(&self) -> &'a E {
        self.source_event
    }
    pub fn entity<'x>(&'x mut self, entity: Entity) -> EntityCommands<'w, 's, 'x> {
        self.elements.commands.entity(entity)
    }
    pub fn load<T: Asset>(&self, path: String) -> Handle<T> {
        self.asset_server.load(path)
    }
    pub fn add<C: Command>(&mut self, command: C) {
        self.elements.commands.add(command);
    }
    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.elements.commands
    }
    pub fn connect<'x>(&'x mut self) -> ConnectCommands<'w, 's, 'x, ()> {
        ConnectCommands {
            commands: &mut self.elements.commands,
            data: (),
        }
    }
    pub fn time(&self) -> &Time {
        self.time_resource
    }
    pub fn send_event<T: Event>(&mut self, event: T) {
        self.elements.commands.add(|world: &mut World| {
            world.resource_mut::<Events<T>>().send(event);
        });
    }
}

impl<'a, 'w, 's, E: Event> Deref for EventContext<'a, 'w, 's, E> {
    type Target = Elements<'w, 's>;
    fn deref(&self) -> &Self::Target {
        self.elements
    }
}

impl<'a, 'w, 's, E: Event> DerefMut for EventContext<'a, 'w, 's, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.elements
    }
}

pub struct Handler<Q: WorldQuery, E: Event>(Box<dyn Fn(&mut EventContext<E>, &mut QueryItem<Q>)>);
impl<Q: 'static + WorldQuery, E: Event> Handler<Q, E> {
    pub fn run(&self, ctx: &mut EventContext<E>, args: &mut QueryItem<Q>) {
        self.0(ctx, args)
    }

    pub fn run_without_target(&self, ctx: &mut EventContext<E>) {
        let empty = &mut () as &mut dyn Any;
        if let Some(empty) = empty.downcast_mut::<QueryItem<Q>>() {
            self.0(ctx, empty)
        } else {
            warn!(
                "Can't invoke eventhandler without target for Handler<{}, {}>",
                type_name::<Q>(),
                type_name::<E>(),
            )
        }
    }
}

unsafe impl<Q: WorldQuery, E: Event> Send for Handler<Q, E> {}
unsafe impl<Q: WorldQuery, E: Event> Sync for Handler<Q, E> {}

pub struct Connection<Q: WorldQuery, E: Event> {
    pub(crate) source: Option<Entity>,
    pub(crate) target: Option<Entity>,
    pub(crate) handler: Handler<Q, E>,
    pub(crate) filter: EventFilter<E>,
}

impl<Q: 'static + WorldQuery, E: Event> Connection<Q, E> {
    // pub fn handles(&self, event: &E) -> bool {
    //     (self.filter)(event)
    // }

    pub fn from(mut self, source: Entity) -> Self {
        self.source = Some(source);
        self
    }

    pub fn write(self, world: &mut World) {
        world
            .resource::<RelationsSystems>()
            .add_signals_processor::<Q, E>();
        let mut connections = world.get_resource_or_insert_with(Connections::<Q, E>::default);
        connections.add(self);
    }
}

impl<Q: 'static + WorldQuery, E: Event> Command for Connection<Q, E> {
    fn apply(self, world: &mut World) {
        self.write(world);
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Connections<Q: WorldQuery, E: Event>(HashMap<EventFilter<E>, EntityConnections<Q, E>>);

impl<Q: WorldQuery, E: Event> Default for Connections<Q, E> {
    fn default() -> Self {
        Connections(HashMap::new())
    }
}

impl<Q: 'static + WorldQuery, E: Event> Connections<Q, E> {
    pub fn process<F: FnMut(&Vec<(Option<Entity>, Handler<Q, E>)>)>(
        &self,
        event: &E,
        mut processor: F,
    ) {
        for (filter, connections) in self.iter() {
            match filter {
                EventFilter::Entity(filter) => {
                    for entity in filter(event) {
                        if let Some(handlers) = connections.get(&Some(entity)) {
                            processor(handlers)
                        }
                    }
                }
                EventFilter::World(filter) if filter(event) => {
                    if let Some(handlers) = connections.get(&None) {
                        processor(handlers)
                    }
                }
                _ => {}
            }
        }
    }
    pub fn add(&mut self, connection: Connection<Q, E>) {
        let source = connection.source;
        let filter = connection.filter;
        let handler = connection.handler;
        let target = if connection.target.is_some() {
            connection.target
        } else {
            source
        };
        if target.is_none() && TypeId::of::<Q>() != TypeId::of::<()>() {
            warn!(
                "Unable to register targetless connection for Handler<{}, {}>",
                type_name::<Q>(),
                type_name::<E>(),
            );
            return;
        }
        let entry = self.0.entry(filter).or_default();
        entry.targets.entry(target).or_default().push(source);
        let handlers = entry.sources.entry(source).or_default();
        handlers.push((target, handler));
        handlers.sort_by_key(|(target, _)| *target);
    }
    pub fn remove(&mut self, entity: &Entity) {
        for connections in self.0.values_mut() {
            if let Some(connections_to) = connections.targets.remove(&Some(*entity)) {
                for connection_to in connections_to.iter() {
                    connections
                        .sources
                        .entry(*connection_to)
                        .and_modify(|e| e.retain(|c| c.0 != Some(*entity)));
                }
            }
            connections.sources.remove(&Some(*entity));
        }
    }

    /// Clear connection entries matched the predicate `func`
    pub fn drain<F: Fn(Entity) -> bool>(&mut self, func: F) {
        for (_, connections) in self.iter_mut() {
            connections.sources.retain(|k, _| {
                if let Some(entity) = k {
                    !func(*entity)
                } else {
                    true
                }
            });

            connections.targets.retain(|entity, targets| {
                let Some(entity) = entity else { return true };
                if !func(*entity) {
                    return true;
                }
                for source in targets.iter() {
                    connections
                        .sources
                        .entry(*source)
                        .and_modify(|e| e.retain(|c| c.0 != Some(*entity)));
                }
                false
            });
        }
    }
}

pub struct EntityConnections<Q: WorldQuery, E: Event> {
    pub(crate) sources: HashMap<Option<Entity>, Vec<(Option<Entity>, Handler<Q, E>)>>,
    pub(crate) targets: HashMap<Option<Entity>, Vec<Option<Entity>>>,
}

impl<Q: WorldQuery, E: Event> Deref for EntityConnections<Q, E> {
    type Target = HashMap<Option<Entity>, Vec<(Option<Entity>, Handler<Q, E>)>>;
    fn deref(&self) -> &Self::Target {
        &self.sources
    }
}

impl<Q: WorldQuery, E: Event> Default for EntityConnections<Q, E> {
    fn default() -> Self {
        EntityConnections {
            sources: Default::default(),
            targets: Default::default(),
        }
    }
}

// commands.add( /* one of */
//  Connect::entity(e).on(btn_pressed).func(|_| { })
//  Connect::entity(e).on(btn_pressed).handle(run!(for e |_| { })
//  Connect::event(mouse_down).to_func(|_| { }) --- ?
//  Connect::event(mosue_down).to_handler(run!(for e |_| {}))
// )
pub struct Connect;
impl Connect {
    pub fn entity(entity: Entity) -> ConnectEntity {
        ConnectEntity(entity)
    }
    pub fn event<E: Event>(filter: WorldEvent<E>) -> ConnectEvent<E> {
        ConnectEvent(EventFilter::World(filter))
    }
}

pub struct ConnectEntity(Entity);
impl ConnectEntity {
    pub fn on<E: Event>(self, filter: EntityEvent<E>) -> ConnectEntityTo<E> {
        ConnectEntityTo(self.0, EventFilter::Entity(filter))
    }
}
pub struct ConnectEvent<E: Event>(EventFilter<E>);
impl<E: Event> ConnectEvent<E> {
    pub fn to_func<F: 'static + Fn(&mut EventContext<E>)>(self, func: F) -> Connection<(), E> {
        Connection {
            target: None,
            source: None,
            filter: self.0,
            handler: Handler(Box::new(move |ctx, _| func(ctx))),
        }
    }
    pub fn to_handler<Q: WorldQuery, F: 'static + Fn(&mut EventContext<E>, &mut QueryItem<Q>)>(
        self,
        (_, target, handler): (PhantomData<Q>, Option<Entity>, F),
    ) -> Connection<Q, E> {
        Connection {
            target,
            source: None,
            filter: self.0,
            handler: Handler(Box::new(handler)),
        }
    }
}

pub struct ConnectEntityTo<E: Event>(Entity, EventFilter<E>);
impl<E: Event> ConnectEntityTo<E> {
    pub fn func<F: 'static + Fn(&mut EventContext<E>)>(self, func: F) -> Connection<(), E> {
        Connection {
            target: None,
            source: Some(self.0),
            filter: self.1,
            handler: Handler(Box::new(move |ctx, _| func(ctx))),
        }
    }
    pub fn handle<Q: WorldQuery, F: 'static + Fn(&mut EventContext<E>, &mut QueryItem<Q>)>(
        self,
        (_, target, handler): (PhantomData<Q>, Option<Entity>, F),
    ) -> Connection<Q, E> {
        Connection {
            target,
            source: Some(self.0),
            filter: self.1,
            handler: Handler(Box::new(handler)),
        }
    }
}

pub trait ConnectCommandsExtension<'w, 's> {
    fn connect<'a>(&'a mut self) -> ConnectCommands<'w, 's, 'a, ()>;
}

impl<'w, 's> ConnectCommandsExtension<'w, 's> for Commands<'w, 's> {
    fn connect<'a>(&'a mut self) -> ConnectCommands<'w, 's, 'a, ()> {
        ConnectCommands {
            commands: self,
            data: (),
        }
    }
}

pub struct ConnectCommands<'w, 's, 'a, T> {
    commands: &'a mut Commands<'w, 's>,
    data: T,
}

impl<'w, 's, 'a> ConnectCommands<'w, 's, 'a, ()> {
    pub fn event<E: Event>(
        self,
        filter: WorldEvent<E>,
    ) -> ConnectCommands<'w, 's, 'a, WorldEvent<E>> {
        ConnectCommands {
            commands: self.commands,
            data: filter,
        }
    }
    pub fn entity(self, entity: Entity) -> ConnectCommands<'w, 's, 'a, Entity> {
        ConnectCommands {
            commands: self.commands,
            data: entity,
        }
    }
}

impl<'w, 's, 'a, E: Event> ConnectCommands<'w, 's, 'a, WorldEvent<E>> {
    pub fn to_func<F: 'static + Fn(&mut EventContext<E>)>(self, func: F) {
        self.commands.add(Connection {
            target: None,
            source: None,
            filter: EventFilter::World(self.data),
            handler: Handler::<(), E>(Box::new(move |ctx, _| func(ctx))),
        })
    }
    pub fn to_handler<
        Q: 'static + WorldQuery,
        F: 'static + Fn(&mut EventContext<E>, &mut QueryItem<Q>),
    >(
        self,
        (_, target, handler): (PhantomData<Q>, Option<Entity>, F),
    ) {
        self.commands.add(Connection {
            target,
            source: None,
            filter: EventFilter::World(self.data),
            handler: Handler(Box::new(handler)),
        });
    }
}

impl<'w, 's, 'a> ConnectCommands<'w, 's, 'a, Entity> {
    pub fn on<E: Event>(
        self,
        filter: EntityEvent<E>,
    ) -> ConnectCommands<'w, 's, 'a, (Entity, EventFilter<E>)> {
        let entity = self.data;
        ConnectCommands {
            commands: self.commands,
            data: (entity, EventFilter::Entity(filter)),
        }
    }
}

impl<'w, 's, 'a, E: Event> ConnectCommands<'w, 's, 'a, (Entity, EventFilter<E>)> {
    pub fn func<F: 'static + Fn(&mut EventContext<E>)>(self, func: F) {
        let (entity, filter) = self.data;
        self.commands.add(Connection {
            filter,
            target: None,
            source: Some(entity),
            handler: Handler::<(), E>(Box::new(move |ctx, _| func(ctx))),
        })
    }

    pub fn handle<
        Q: 'static + WorldQuery,
        F: 'static + Fn(&mut EventContext<E>, &mut QueryItem<Q>),
    >(
        self,
        (_, target, handler): (PhantomData<Q>, Option<Entity>, F),
    ) {
        let (entity, filter) = self.data;
        self.commands.add(Connection {
            target,
            filter,
            source: Some(entity),
            handler: Handler(Box::new(handler)),
        })
    }
}
