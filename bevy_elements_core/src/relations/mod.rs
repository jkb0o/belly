pub mod bound;
mod connect;
pub mod transform;

use std::{
    any::TypeId,
    sync::{Arc, RwLock},
};

use bevy::{prelude::*, utils::HashSet};

use self::bound::{process_binds, watch_changes, BindableSource, BindableTarget, ChangesState};
pub use self::connect::{
    Connect, ConnectionEntityContext, ConnectionGeneralContext, ConnectionTo, Connections, Signal,
};

pub struct RelationsPlugin;

impl Plugin for RelationsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChangesState>();
        app.add_system_to_stage(CoreStage::PreUpdate, process_relations_system);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum BindingStage {
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
    let systems_ref = world
        .get_resource_or_insert_with(RelationsSystems::default)
        .clone();
    let mut systems = systems_ref.0.write().unwrap();
    systems.run(world);
}

pub fn process_signals_system<C: Component, S: Signal>(
    asset_server: Res<AssetServer>,
    connections: Res<Connections<C, S>>,
    time: Res<Time>,
    mut commands: Commands,
    mut events: EventReader<S>,
    mut components: Query<&mut C>,
) {
    for signal in events.iter() {
        for source in signal.sources().iter() {
            if let Some(connections) = connections.get(&source) {
                let mut context = ConnectionGeneralContext {
                    source_event: signal,
                    source: *source,
                    time_resource: &time,
                    asset_server: asset_server.clone(),
                    commands: &mut commands,
                };
                for connection in connections.iter().filter(|c| c.handles(signal)) {
                    match &connection.target {
                        ConnectionTo::General { handler } => {
                            handler(&mut context);
                        }
                        ConnectionTo::Entity { target, handler } => {
                            let mut entity_context = ConnectionEntityContext {
                                target: *target,
                                ctx: &mut context,
                            };
                            handler(&mut entity_context);
                        }
                        ConnectionTo::Component { target, handler } => {
                            if let Ok(mut component) = components.get_mut(*target) {
                                let mut entity_context = ConnectionEntityContext {
                                    target: *target,
                                    ctx: &mut context,
                                };
                                handler(&mut entity_context, &mut component);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn cleanup_signals_system<C: Component, S: Signal>(
    mut connections: ResMut<Connections<C, S>>,
    mut commands: Commands,
) {
    let entities_to_remove = connections
        .entities()
        .filter(|e| commands.get_entity(*e).is_none())
        .collect::<Vec<_>>();
    entities_to_remove
        .iter()
        .for_each(|e| connections.remove(e));
}

pub(crate) struct BindingSystemsInternal {
    schedule: Schedule,
    processors: HashSet<(TypeId, TypeId)>,
    custom: HashSet<TypeId>,

    // new `bound` added system hashes
    systems: HashSet<(TypeId, TypeId, TypeId, TypeId)>,
    watchers: HashSet<TypeId>,
}

#[derive(Default, Clone, Resource)]
pub struct RelationsSystems(pub(crate) Arc<RwLock<BindingSystemsInternal>>);

impl BindingSystemsInternal {
    pub fn add_signals_processor<C: Component, S: Signal>(&mut self) {
        let entry = (TypeId::of::<C>(), TypeId::of::<S>());
        if self.processors.contains(&entry) {
            return;
        }
        self.processors.insert(entry);
        self.schedule
            .add_system_to_stage(BindingStage::Process, process_signals_system::<C, S>);
        self.schedule
            .add_system_to_stage(BindingStage::Process, cleanup_signals_system::<C, S>);
    }
    pub fn add_custom_system<Params, S: IntoSystemDescriptor<Params>>(
        &mut self,
        system_id: TypeId,
        system: S,
    ) {
        if self.custom.contains(&system_id) {
            return;
        }
        self.custom.insert(system_id);
        self.schedule
            .add_system_to_stage(BindingStage::Custom, system);
    }
    pub fn run(&mut self, world: &mut World) {
        let mut last_state = world.resource::<ChangesState>().get();
        loop {
            self.schedule.run(world);
            let current_state = world.resource::<ChangesState>().get();
            if last_state == current_state {
                break;
            } else {
                last_state = current_state;
            }
        }
    }

    fn add_bind_system<R: Component, W: Component, S: BindableSource, T: BindableTarget>(
        &mut self,
    ) {
        let watcher = TypeId::of::<R>();
        let entry = (
            TypeId::of::<W>(),
            TypeId::of::<W>(),
            TypeId::of::<S>(),
            TypeId::of::<T>(),
        );
        if !self.watchers.contains(&watcher) {
            self.watchers.insert(watcher);
            self.schedule
                .add_system_to_stage(BindingStage::Watch, watch_changes::<R>);
        }

        if !self.systems.contains(&entry) {
            self.systems.insert(entry);
            self.schedule
                .add_system_to_stage(BindingStage::Bind, process_binds::<R, W, S, T>);
        }
    }
}

impl Default for BindingSystemsInternal {
    fn default() -> Self {
        let processors = HashSet::default();
        let custom = HashSet::default();

        // new `bound` hashes
        let systems = HashSet::default();
        let watchers = HashSet::default();

        let mut schedule = Schedule::default();
        schedule
            .add_stage(BindingStage::Process, SystemStage::parallel())
            .add_stage(BindingStage::Collect, SystemStage::parallel())
            .add_stage(BindingStage::Apply, SystemStage::parallel())
            .add_stage(BindingStage::Custom, SystemStage::parallel())
            .add_stage(BindingStage::Report, SystemStage::parallel())
            // new `bound` stages
            .add_stage(BindingStage::Bind, SystemStage::parallel())
            .add_stage(BindingStage::Watch, SystemStage::parallel());
        Self {
            schedule,
            processors,
            custom,

            // new `bound` hashes
            systems,
            watchers,
        }
    }
}
