pub mod bind;
pub mod connect;
pub mod ops;
pub mod props;

use crate::element::Elements;

use self::bind::{BindableSource, BindableTarget, ChangesState};
pub use self::connect::{Connections, EventContext, Handler};
use bevy::{
    ecs::{entity::Entities, event::Event, query::WorldQuery},
    log::Level,
    prelude::*,
    utils::{tracing::span, HashSet},
};
use itertools::Itertools;
use std::{
    any::TypeId,
    mem,
    sync::{Arc, RwLock},
};

pub struct RelationsPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum RelationsSet {
    PreUpdate,
    Update,
    PostUpdate,
}

impl Plugin for RelationsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RelationsSystems>();
        app.init_resource::<ChangesState>();
        app.configure_set(Update, RelationsSet::PreUpdate);
        app.configure_set(PostUpdate, RelationsSet::Update);
        app.configure_set(Last, RelationsSet::PostUpdate);

        app.add_systems(
            PreUpdate,
            process_relations_system.in_set(RelationsSet::PreUpdate),
        );
        // For some reason with bevy 0.10 I can't process this system multiple times,
        // App panics with:
        // '`"Update"` and `"PostUpdate"` have a `before`-`after` relationship (which
        // may be transitive) but share systems.
        // app .add_systems(process_relations_system.in_set(RelationsSet::PostUpdate));
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum BindingSet {
    Process,
    Collect,
    Apply,
    Custom,
    Report,

    // new `bound` system states
    Bind,
    Watch,
}

pub fn process_relations_system(world: &mut World) {
    let relations = world.resource::<RelationsSystems>().clone();
    relations.run(world);
}
pub fn process_relations_system_b(world: &mut World) {
    let relations = world.resource::<RelationsSystems>().clone();
    relations.run(world);
}

pub fn process_signals_system<P: 'static + WorldQuery, E: Event>(
    asset_server: Res<AssetServer>,
    connections: Res<Connections<P, E>>,
    time: Res<Time>,
    mut elements: Elements,
    mut events: EventReader<E>,
    mut components: Query<P>,
) {
    for signal in events.iter() {
        let mut context = EventContext {
            source_event: signal,
            time_resource: &time,
            asset_server: asset_server.clone(),
            elements: &mut elements,
        };
        connections.process(signal, |handlers| {
            for (target, group) in &handlers.iter().group_by(|(target, _)| target) {
                if let Some(target) = target {
                    let Ok(mut args) = components.get_mut(*target) else {
                        continue
                    };
                    for (_, handler) in group {
                        handler.run(&mut context, &mut args);
                    }
                } else {
                    for (_, handler) in group {
                        handler.run_without_target(&mut context);
                    }
                }
            }
        });
    }
}

pub fn cleanup_signals_system<P: 'static + WorldQuery, E: Event>(
    mut connections: ResMut<Connections<P, E>>,
    entities: &Entities,
) {
    connections.drain(|e| !entities.contains(e));
}
#[derive(Default, Clone, Resource, Deref)]
pub struct RelationsSystems(pub(crate) Arc<BindingSystemsInternal>);
unsafe impl Send for RelationsSystems {}
unsafe impl Sync for RelationsSystems {}

pub struct BindingSystemsInternal {
    schedule: RwLock<Schedule>,
    system_queue: RwLock<Vec<Box<dyn FnOnce(&mut Schedule)>>>,
    processors: RwLock<HashSet<(TypeId, TypeId)>>,
    custom: RwLock<HashSet<TypeId>>,

    // new `bound` added system hashes
    systems: RwLock<HashSet<(TypeId, TypeId, TypeId, TypeId)>>,
    watchers: RwLock<HashSet<TypeId>>,
}

impl BindingSystemsInternal {
    pub fn add_signals_processor<P: 'static + WorldQuery, E: Event>(&self) {
        let entry = (TypeId::of::<P>(), TypeId::of::<E>());
        if self.processors.read().unwrap().contains(&entry) {
            return;
        }
        let mut processors = self.processors.write().unwrap();
        if processors.contains(&entry) {
            return;
        }
        processors.insert(entry);
        self.system_queue
            .write()
            .unwrap()
            .push(Box::new(|schedule| {
                schedule.add_systems(process_signals_system::<P, E>.in_set(BindingSet::Process));
                schedule.add_systems(cleanup_signals_system::<P, E>.in_set(BindingSet::Process));
            }));
    }
    pub fn add_custom_system<Params, S: 'static + IntoSystemConfigs<Params>>(
        &self,
        system_id: TypeId,
        system: S,
    ) {
        if self.custom.read().unwrap().contains(&system_id) {
            return;
        }
        let mut custom = self.custom.write().unwrap();
        if custom.contains(&system_id) {
            return;
        }
        custom.insert(system_id);
        self.system_queue
            .write()
            .unwrap()
            .push(Box::new(move |schedule| {
                schedule.add_systems(system.in_set(BindingSet::Custom));
            }));
    }
    pub fn run(&self, world: &mut World) {
        let span = span!(Level::INFO, "belly");
        let _enter = span.enter();
        let mut last_state = world.resource::<ChangesState>().get();
        loop {
            self.schedule.write().unwrap().run(world);
            {
                let mut queue = self.system_queue.write().unwrap();
                let mut schedule = self.schedule.write().unwrap();
                for add_system in mem::take(&mut *queue) {
                    add_system(&mut schedule)
                }
            }
            let current_state = world.resource::<ChangesState>().get();
            if last_state == current_state {
                break;
            } else {
                last_state = current_state;
            }
        }
    }

    fn add_component_to_component<
        R: Component,
        W: Component,
        S: BindableSource,
        T: BindableTarget,
    >(
        &self,
    ) {
        let watcher = TypeId::of::<R>();
        let entry = (
            TypeId::of::<R>(),
            TypeId::of::<W>(),
            TypeId::of::<S>(),
            TypeId::of::<T>(),
        );
        if self.watchers.read().unwrap().contains(&watcher) {
            return;
        }
        {
            let watchers = self.watchers.write().unwrap();
            if watchers.contains(&watcher) {
                return;
            }
            self.system_queue
                .write()
                .unwrap()
                .push(Box::new(|schedule| {
                    schedule.add_systems(bind::watch_changes::<R>.in_set(BindingSet::Watch));
                }));
        }

        if self.systems.read().unwrap().contains(&entry) {
            return;
        }
        let mut systems = self.systems.write().unwrap();
        if systems.contains(&entry) {
            return;
        }
        systems.insert(entry);
        self.system_queue
            .write()
            .unwrap()
            .push(Box::new(|schedule| {
                schedule.add_systems(
                    bind::component_to_component_system::<R, W, S, T>.in_set(BindingSet::Bind),
                );
            }));
    }
    fn add_resource_to_component<
        R: Resource,
        W: Component,
        S: BindableSource,
        T: BindableTarget,
    >(
        &self,
    ) {
        let entry = (
            TypeId::of::<W>(),
            TypeId::of::<W>(),
            TypeId::of::<S>(),
            TypeId::of::<T>(),
        );
        if self.systems.read().unwrap().contains(&entry) {
            return;
        }
        let mut systems = self.systems.write().unwrap();
        if systems.contains(&entry) {
            return;
        }
        systems.insert(entry);
        self.system_queue
            .write()
            .unwrap()
            .push(Box::new(|schedule| {
                schedule.add_systems(
                    bind::resource_to_component_system::<R, W, S, T>.in_set(BindingSet::Bind),
                );
            }));
    }
}

impl Default for BindingSystemsInternal {
    fn default() -> Self {
        let processors = HashSet::default();
        let custom = HashSet::default();

        // new `bound` hashes
        let systems = HashSet::default();
        let watchers = HashSet::default();

        let schedule = Schedule::default();
        Self {
            schedule: RwLock::new(schedule),
            processors: RwLock::new(processors),
            custom: RwLock::new(custom),

            // new `bound` hashes
            systems: RwLock::new(systems),
            watchers: RwLock::new(watchers),
            system_queue: RwLock::new(vec![]),
        }
    }
}
