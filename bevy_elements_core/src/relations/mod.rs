mod bind;
mod connect;

use std::{
    any::TypeId,
    sync::{Arc, RwLock},
};

use bevy::{prelude::*, utils::HashSet};

pub use self::{
    bind::{
        Bind, BindFrom, BindFromUntyped, BindTo, BindToUntyped, BindValue, BindingChanges,
        BindingSource, BindingTarget, ChangeCounter, Changes,
    },
    connect::{
        Connect, ConnectionEntityContext, ConnectionGeneralContext, ConnectionTo, Connections,
        Signal,
    },
};

pub struct RelationsPlugin;

impl Plugin for RelationsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChangeCounter>()
            .add_system_to_stage(CoreStage::PreUpdate, process_relations_system);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum BindingStage {
    Process,
    Collect,
    Apply,
    Custom,
    Report,
}

pub fn process_relations_system(world: &mut World) {
    let systems_ref = world
        .get_resource_or_insert_with(RelationsSystems::default)
        .clone();
    let mut systems = systems_ref.0.write().unwrap();
    systems.run(world);
}

pub fn collect_changes_system<R: Component, T: BindValue>(
    mut change_detector: ResMut<Changes<R>>,
    sources: Query<(&R, &BindingSource<R, T>), Changed<R>>,
    mut changes: Query<&mut BindingChanges<T>>,
) {
    // panic!("won't run");
    if !sources.is_empty() {
        change_detector.set_changed();
    }
    for (source_component, source) in sources.iter() {
        for (entity, reader, writer_id) in source.iter() {
            let value = reader(source_component);
            if let Ok(mut changes) = changes.get_mut(*entity) {
                changes.insert(*writer_id, value.clone());
            }
        }
    }
}

pub fn apply_changes_system<W: Component, T: BindValue>(
    mut changes: Query<
        (&BindingChanges<T>, &BindingTarget<W, T>, &mut W),
        Changed<BindingChanges<T>>,
    >,
) {
    for (changes, target, mut target_component) in changes.iter_mut() {
        for (writer_id, value) in changes.iter() {
            if let Some((equals, write)) = target.get(writer_id) {
                if !equals(&target_component, value) {
                    write(&mut target_component, value);
                }
            }
        }
    }
}

pub fn report_changes_system<R: Component>(
    mut any_changes: ResMut<ChangeCounter>,
    changes: Res<Changes<R>>,
) {
    if changes.is_changed() {
        any_changes.add();
    }
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
    last_writer: usize,
    schedule: Schedule,
    collectors: HashSet<(TypeId, TypeId)>,
    appliers: HashSet<(TypeId, TypeId)>,
    reporters: HashSet<TypeId>,
    processors: HashSet<(TypeId, TypeId)>,
    custom: HashSet<TypeId>,
}

#[derive(Default, Clone, Resource)]
pub struct RelationsSystems(pub(crate) Arc<RwLock<BindingSystemsInternal>>);

impl BindingSystemsInternal {
    fn reserve_writer(&mut self) -> usize {
        let writer_idx = self.last_writer;
        self.last_writer += 1;
        writer_idx
    }

    fn add_collect_system<R: Component, T: BindValue>(&mut self) -> bool {
        let reader = TypeId::of::<R>();
        let entry = (reader, TypeId::of::<T>());
        if self.collectors.contains(&entry) {
            return false;
        }
        self.collectors.insert(entry);
        self.schedule
            .add_system_to_stage(BindingStage::Collect, collect_changes_system::<R, T>);
        if self.reporters.contains(&reader) {
            false
        } else {
            self.schedule
                .add_system_to_stage(BindingStage::Report, report_changes_system::<R>);
            self.reporters.insert(reader);
            true
        }
    }
    fn add_apply_system<W: Component, T: BindValue>(&mut self) {
        let entry = (TypeId::of::<W>(), TypeId::of::<T>());
        if self.appliers.contains(&entry) {
            return;
        }
        self.appliers.insert(entry);
        self.schedule
            .add_system_to_stage(BindingStage::Apply, apply_changes_system::<W, T>);
    }
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
        let mut last_change = world.resource::<ChangeCounter>().get();
        loop {
            self.schedule.run(world);
            let current_change = world.resource::<ChangeCounter>().get();
            if last_change == current_change {
                break;
            } else {
                last_change = current_change;
            }
        }
    }
}

impl Default for BindingSystemsInternal {
    fn default() -> Self {
        let collectors = HashSet::default();
        let appliers = HashSet::default();
        let reporters = HashSet::default();
        let processors = HashSet::default();
        let custom = HashSet::default();
        let mut schedule = Schedule::default();
        schedule
            .add_stage(BindingStage::Process, SystemStage::parallel())
            .add_stage(BindingStage::Collect, SystemStage::parallel())
            .add_stage(BindingStage::Apply, SystemStage::parallel())
            .add_stage(BindingStage::Custom, SystemStage::parallel())
            .add_stage(BindingStage::Report, SystemStage::parallel());
        Self {
            schedule,
            collectors,
            appliers,
            reporters,
            processors,
            custom,
            last_writer: 0,
        }
    }
}
