use bevy::{
    asset::Asset,
    ecs::{
        event::Event,
        system::{Command, EntityCommands},
    },
    prelude::*,
    utils::{HashMap, HashSet},
};
use std::{
    any::{type_name, TypeId},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
};

use crate::{ElementsBuilder, PointerInput, WithElements};

pub struct BindPlugin;

impl Plugin for BindPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChangeCounter>()
            .add_system_to_stage(CoreStage::PreUpdate, process_binds_system);
    }
}

pub trait BindValue: Default + PartialEq + Clone + Send + Sync + 'static {}
impl<T: Default + PartialEq + Clone + Send + Sync + 'static> BindValue for T {}

#[derive(Component)]
pub struct BindingChanges<T: BindValue>(HashMap<usize, T>);

impl<T: BindValue> BindingChanges<T> {
    pub fn new() -> BindingChanges<T> {
        BindingChanges(default())
    }
}

#[derive(Component)]
pub struct BindingSource<R: Component, T: BindValue>(Vec<(Entity, fn(&R) -> T, usize)>);

impl<R: Component, T: BindValue> BindingSource<R, T> {
    pub fn new() -> BindingSource<R, T> {
        BindingSource(default())
    }
}

#[derive(Component, Default)]
pub struct BindingTarget<W: Component, T: BindValue>(
    HashMap<usize, (fn(&W, &T) -> bool, fn(&mut W, &T))>,
);

impl<W: Component, T: BindValue> BindingTarget<W, T> {
    fn new() -> BindingTarget<W, T> {
        BindingTarget(default())
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

#[derive(Resource)]
pub struct Changes<T: Component>(PhantomData<T>);
impl<T: Component> Default for Changes<T> {
    fn default() -> Self {
        Changes::<T>(PhantomData::<T>)
    }
}
#[derive(Resource, Default)]
pub struct ChangeCounter(usize);

pub fn process_binds_system(world: &mut World) {
    let systems_ref = world
        .get_resource_or_insert_with(BindingSystems::default)
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
        for (entity, reader, writer_id) in source.0.iter() {
            let value = reader(source_component);
            if let Ok(mut changes) = changes.get_mut(*entity) {
                changes.0.insert(*writer_id, value.clone());
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
        for (writer_id, value) in changes.0.iter() {
            if let Some((equals, write)) = target.0.get(writer_id) {
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
        any_changes.0 += 1;
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
            if let Some(connections) = connections.map.get(&source) {
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
        .map
        .keys()
        .chain(connections.index.keys())
        .filter(|e| commands.get_entity(**e).is_none())
        .map(|e| *e)
        .collect::<HashSet<_>>();
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
pub struct BindingSystems(pub(crate) Arc<RwLock<BindingSystemsInternal>>);

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
        let mut last_change = world.resource::<ChangeCounter>().0;
        loop {
            self.schedule.run(world);
            let current_change = world.resource::<ChangeCounter>().0;
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

pub trait Signal: Event {
    fn sources(&self) -> &[Entity];
}

pub struct ConnectionGeneralContext<'a, 'w, 's, S: Signal> {
    source_event: &'a S,
    source: Entity,
    time_resource: &'a Time,
    asset_server: AssetServer,
    commands: &'a mut Commands<'w, 's>,
}

impl<'a, 'w, 's, S: Signal> ConnectionGeneralContext<'a, 'w, 's, S> {
    pub fn event(&self) -> &S {
        self.source_event
    }
    pub fn source<'x>(&'x mut self) -> EntityCommands<'w, 's, 'x> {
        let source = self.source;
        self.commands.entity(source)
    }
    pub fn load<T: Asset>(&self, path: &str) -> Handle<T> {
        self.asset_server.load(path)
    }
    pub fn add<C: Command>(&mut self, command: C) {
        self.commands.add(command);
    }
    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.commands
    }
    pub fn time(&self) -> &Time {
        self.time_resource
    }
}

pub struct ConnectionEntityContext<'a, 'w, 's, 'c, S: Signal> {
    target: Entity,
    ctx: &'c mut ConnectionGeneralContext<'a, 'w, 's, S>,
}

impl<'a, 'w, 's, 'c, S: Signal> ConnectionEntityContext<'a, 'w, 's, 'c, S> {
    pub fn target<'x>(&'x mut self) -> EntityCommands<'w, 's, 'x> {
        let target = self.target;
        self.commands.entity(target)
    }

    pub fn render(&mut self, eml: ElementsBuilder) {
        self.target().with_elements(eml);
    }

    pub fn replace(&mut self, eml: ElementsBuilder) {
        self.target().despawn_descendants();
        self.target().with_elements(eml);
    }
}

impl<'a, 'w, 's, 'c, S: Signal> Deref for ConnectionEntityContext<'a, 'w, 's, 'c, S> {
    type Target = ConnectionGeneralContext<'a, 'w, 's, S>;
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<'a, 'w, 's, 'c, S: Signal> DerefMut for ConnectionEntityContext<'a, 'w, 's, 'c, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

pub enum ConnectionTo<C: Component, S: Signal> {
    General {
        handler: fn(&mut ConnectionGeneralContext<S>),
    },
    Entity {
        target: Entity,
        handler: fn(&mut ConnectionEntityContext<S>),
    },
    Component {
        target: Entity,
        handler: fn(&mut ConnectionEntityContext<S>, &mut Mut<C>),
    },
}

#[derive(Component)]
pub struct WithoutComponent;

impl<C: Component, S: Signal> ConnectionTo<C, S> {
    pub fn component(
        target: Entity,
        handler: fn(&mut ConnectionEntityContext<S>, &mut Mut<C>),
    ) -> ConnectionTo<C, S> {
        ConnectionTo::Component { target, handler }
    }

    pub fn filter(self, filter: fn(&S) -> bool) -> Connection<C, S> {
        Connection {
            target: self,
            filter,
        }
    }

    pub fn id(&self) -> Option<Entity> {
        match self {
            ConnectionTo::Component { target, handler: _ } => Some(*target),
            ConnectionTo::Entity { target, handler: _ } => Some(*target),
            _ => None,
        }
    }
}

impl<S: Signal> ConnectionTo<WithoutComponent, S> {
    pub fn entity(
        target: Entity,
        handler: fn(&mut ConnectionEntityContext<S>),
    ) -> ConnectionTo<WithoutComponent, S> {
        ConnectionTo::Entity { target, handler }
    }

    pub fn general(
        handler: fn(&mut ConnectionGeneralContext<S>),
    ) -> ConnectionTo<WithoutComponent, S> {
        ConnectionTo::General { handler }
    }
}

pub struct Connection<C: Component, S: Signal> {
    target: ConnectionTo<C, S>,
    filter: fn(&S) -> bool,
}

impl<C: Component, S: Signal> Connection<C, S> {
    fn handles(&self, signal: &S) -> bool {
        (self.filter)(signal)
    }

    pub fn from(self, source: Entity) -> Connect<C, S> {
        Connect {
            source,
            target: self,
        }
    }
}

impl Signal for PointerInput {
    fn sources(&self) -> &[Entity] {
        &self.entities
    }
}

pub struct Connect<C: Component, S: Signal> {
    source: Entity,
    target: Connection<C, S>,
}

impl<C: Component, S: Signal> Connect<C, S> {
    pub fn write(self, world: &mut World) {
        {
            let systems = world.get_resource_or_insert_with(BindingSystems::default);
            systems.0.write().unwrap().add_signals_processor::<C, S>();
        }
        {
            let mut connections = world.get_resource_or_insert_with(Connections::<C, S>::default);
            connections.add(self);
        }
    }
}

// impl<C: Component, S: Signal> Command for Connect<C, S> { }

#[derive(Resource)]
pub struct Connections<C: Component, S: Signal> {
    map: HashMap<Entity, Vec<Connection<C, S>>>,
    index: HashMap<Entity, Vec<Entity>>,
}

impl<C: Component, S: Signal> Default for Connections<C, S> {
    fn default() -> Self {
        Connections {
            map: Default::default(),
            index: Default::default(),
        }
    }
}

impl<C: Component, S: Signal> Connections<C, S> {
    pub fn add(&mut self, connection: Connect<C, S>) {
        if let Some(target) = connection.target.target.id() {
            self.index
                .entry(target)
                .or_default()
                .push(connection.source)
        }
        self.map
            .entry(connection.source)
            .or_default()
            .push(connection.target);
    }
    pub fn remove(&mut self, source: &Entity) {
        if let Some(connections_to) = self.index.remove(source) {
            for connection_to in connections_to.iter() {
                self.map
                    .entry(*connection_to)
                    .and_modify(|e| e.retain(|c| c.target.id() != Some(*source)));
            }
        }
        self.map.remove(&source);
    }
}

#[derive(Component)]
pub struct Handlers<C: Component>(Vec<fn(&mut C)>);

pub struct BindFrom<R: Component, T: BindValue> {
    source: Entity,
    reader: fn(&R) -> T,
}

impl<R: Component, T: BindValue> BindFrom<R, T> {
    pub fn new(source: Entity, reader: fn(&R) -> T) -> BindFrom<R, T> {
        BindFrom { source, reader }
    }

    pub fn write(&self, world: &mut World, target: Entity, writer_id: usize) {
        {
            let systems = world.get_resource_or_insert_with(BindingSystems::default);
            if systems.0.write().unwrap().add_collect_system::<R, T>() {
                world.init_resource::<Changes<R>>();
            }
        }
        let mut source_entity = world.entity_mut(self.source);
        if let Some(mut source_component) = source_entity.get_mut::<BindingSource<R, T>>() {
            source_component.0.push((target, self.reader, writer_id));
        } else {
            source_entity.insert(BindingSource(vec![(target, self.reader, writer_id)]));
        }
    }

    pub fn to_untyped(self) -> BindFromUntyped {
        BindFromUntyped::from_typed(self)
    }
}

pub struct BindTo<W: Component, T: BindValue> {
    target: Entity,
    comparer: fn(&W, &T) -> bool,
    writer: fn(&mut W, &T),
}

impl<W: Component, T: BindValue> BindTo<W, T> {
    pub fn new(
        target: Entity,
        comparer: fn(&W, &T) -> bool,
        writer: fn(&mut W, &T),
    ) -> BindTo<W, T> {
        BindTo {
            target,
            comparer,
            writer,
        }
    }

    pub fn write(&self, world: &mut World) -> usize {
        let writer_id = {
            let systems_ref = world.get_resource_or_insert_with(BindingSystems::default);
            let mut systems = systems_ref.0.write().unwrap();
            systems.add_apply_system::<W, T>();
            systems.reserve_writer()
        };

        let mut target_entity = world.entity_mut(self.target);
        if let Some(mut changes_component) = target_entity.get_mut::<BindingChanges<T>>() {
            changes_component.0.insert(writer_id, T::default());
        } else {
            let mut changes = BindingChanges::<T>::new();
            changes.0.insert(writer_id, T::default());
            target_entity.insert(changes);
        }
        if let Some(mut writer_component) = target_entity.get_mut::<BindingTarget<W, T>>() {
            writer_component
                .0
                .insert(writer_id, (self.comparer, self.writer));
        } else {
            let mut writers = BindingTarget::<W, T>::new();
            writers.0.insert(writer_id, (self.comparer, self.writer));
            target_entity.insert(writers);
        }
        writer_id
    }

    pub fn to_untyped(self) -> BindToUntyped {
        BindToUntyped::from_typed(self)
    }
}

pub struct Bind<R: Component, T: BindValue, W: Component> {
    source: BindFrom<R, T>,
    target: BindTo<W, T>,
}

// unsafe impl Send for Bind { }
// unsafe impl Sync for Bind { }

impl<R: Component, T: BindValue, W: Component> Bind<R, T, W> {
    pub fn new(source: BindFrom<R, T>, target: BindTo<W, T>) -> Bind<R, T, W> {
        Bind { source, target }
    }

    pub fn build(
        source: Entity,
        reader: fn(&R) -> T,
        target: Entity,
        comparer: fn(&W, &T) -> bool,
        writer: fn(&mut W, &T),
    ) -> Bind<R, T, W> {
        let source = BindFrom::new(source, reader);
        let target = BindTo::new(target, comparer, writer);
        Self::new(source, target)
    }

    pub fn to_untyped(self) -> BindUntyped {
        BindUntyped {
            bind_from: self.source.to_untyped(),
            bind_to: self.target.to_untyped(),
        }
    }
}

impl<R: Component, T: BindValue, W: Component> Command for Bind<R, T, W> {
    fn write(self, world: &mut World) {
        let writer_id = self.target.write(world);
        self.source.write(world, self.target.target, writer_id);
    }
}

type UntypedWriteDescriptor = (Entity, usize, TypeId, &'static str);
pub struct BindFromUntyped(Box<dyn Fn(&mut World, UntypedWriteDescriptor)>);

impl BindFromUntyped {
    pub fn from_typed<R: Component, T: BindValue>(bind_from: BindFrom<R, T>) -> BindFromUntyped {
        let reader_type = TypeId::of::<T>();
        let reader_type_name = type_name::<T>();
        BindFromUntyped(Box::new(
            move |world, (target, writer_id, writer_type, writer_type_name)| {
                if reader_type == writer_type {
                    bind_from.write(world, target, writer_id);
                } else {
                    error!(
                        "Bind type mismatch: excepted: {}, received: {}",
                        reader_type_name, writer_type_name
                    );
                }
            },
        ))
    }

    pub fn write(&self, world: &mut World, descriptor: UntypedWriteDescriptor) {
        (self.0)(world, descriptor);
    }

    pub fn to<W: Component, T: BindValue>(self, bind: BindTo<W, T>) -> BindUntyped {
        BindUntyped {
            bind_from: self,
            bind_to: bind.to_untyped(),
        }
    }
}

pub struct BindToUntyped(Box<dyn Fn(&mut World) -> UntypedWriteDescriptor>);

impl BindToUntyped {
    pub fn from_typed<W: Component, T: BindValue>(bind_to: BindTo<W, T>) -> BindToUntyped {
        let writer_type = TypeId::of::<T>();
        let writer_type_name = type_name::<T>();
        BindToUntyped(Box::new(move |world| {
            let writer_id = bind_to.write(world);
            (bind_to.target, writer_id, writer_type, writer_type_name)
        }))
    }

    pub fn write(&self, world: &mut World) -> UntypedWriteDescriptor {
        (self.0)(world)
    }

    pub fn from<R: Component, T: BindValue>(self, bind: BindFrom<R, T>) -> BindUntyped {
        BindUntyped {
            bind_from: bind.to_untyped(),
            bind_to: self,
        }
    }
}

pub struct BindUntyped {
    bind_from: BindFromUntyped,
    bind_to: BindToUntyped,
}

unsafe impl Send for BindUntyped {}
unsafe impl Sync for BindUntyped {}

impl Command for BindUntyped {
    fn write(self, world: &mut World) {
        let descriptor = self.bind_to.write(world);
        self.bind_from.write(world, descriptor);
    }
}

#[macro_export]
macro_rules! bind {

    // -------------------------------
    // compile-time protected bindings
    // -------------------------------

    // bind value-to-value
    // bind!(player, Health.value, healthbar, ProgressBar.value)
    (
        $s_entity:expr, $s_class:ident$(.$s_prop:ident)+ =>
        $t_entity:expr, $t_class:ident$(.$t_prop:ident)+
    ) => {
        $crate::bind::Bind::build(
            $s_entity.clone(),
            |s: &$s_class| { s$(.$s_prop)+.clone() },
            $t_entity.clone(),
            |t: &$t_class, v| &t$(.$t_prop)+ == v,
            |t: &mut $t_class, v| { t$(.$t_prop)+.clone_from(v); }
        )
    };

    (
        $s_entity:expr, $s_class:ident$(.$s_prop:ident)+ =>
        $t_entity:expr, $t_class:ident$(.$t_prop:ident$([$t_idx:literal])?)+
    ) => {
        $crate::bind::Bind::build(
            $s_entity.clone(),
            |s: &$s_class| { s$(.$s_prop)+.clone() },
            $t_entity.clone(),
            |t: &$t_class, v| &t$(.$t_prop$([$t_idx])?)+ == v,
            |t: &mut $t_class, v| { t$(.$t_prop$([$t_idx])?)+.clone_from(v); }
        )
    };

    // bind getter-to-value
    // bind!(source, Sprite.color:a, target, Transform.translation.x)
    (
        $s_entity:expr, $s_class:ident$(.$s_prop:ident)+:$s_getter:ident =>
        $t_entity:expr, $t_class:ident$(.$t_prop:ident)+
    ) => {
        $crate::bind::Bind::build(
            $s_entity.clone(),
            |s: &$s_class| { s$(.$s_prop)+.$s_getter().clone() },
            $t_entity.clone(),
            |t: &$t_class, v| &t$(.$t_prop)+ == v,
            |t: &mut $t_class, v| { t$(.$t_prop)+.clone_from(v); }
        )
    };

    // bind value-to-setter
    // bind!(source, Health.value, icon, Sprite.color:r:set_r)
    (
        $s_entity:expr, $s_class:ident$(.$s_prop:ident)+ =>
        $t_entity:expr, $t_class:ident$(.$t_prop:ident)+:$t_getter:ident:$t_setter:ident
    ) => {
        $crate::bind::Bind::build(
            $s_entity.clone(),
            |s: &$s_class| { s$(.$s_prop)+.clone() },
            $t_entity.clone(),
            |t: &$t_class, v| &t$(.$t_prop)+.$t_getter() == v,
            |t: &mut $t_class, v| { t.$($t_prop)+.$t_setter(v.clone()); }
        )
    };

    // ----------------------------------
    // runtime-checkable sources bindings
    // ----------------------------------

    // bind source by value
    // bind!(player, Health.value ->)
    (
        $s_entity:expr, $s_class:ident$(.$s_prop:ident)+ =>
    ) => {
        $crate::bind::BindFrom::new(
            $s_entity.clone(),
            |s: &$s_class| { s$(.$s_prop)+.clone() },
        )
    };
    (
        <= $s_entity:expr, $s_class:ident$(.$s_prop:ident)+
    ) => {
        $crate::bind::BindFrom::new(
            $s_entity.clone(),
            |s: &$s_class| { s$(.$s_prop)+.clone() },
        )
    };
    // bind source by getter
    // bind!(player, Health.color:a ->)
    (
        $s_entity:expr, $s_class:ident$(.$s_prop:ident)+:$s_getter:ident =>
    ) => {
        $crate::bind::BindFrom::new(
            $s_entity.clone(),
            |s: &$s_class| { s$(.$s_prop)+.$s_getter().clone() },
        )
    };

    // ----------------------------------
    // runtime-checkable target bindings
    // ----------------------------------

    // bind target by value
    // bind!(-> player, Health.value)
    (
        => $t_entity:expr, $t_class:ident$(.$t_prop:ident$([$t_idx:literal])?)+
    ) => {
        $crate::bind::BindTo::new(
            $t_entity.clone(),
            |t: &$t_class, v| &t$(.$t_prop$([$t_idx])?)+ == v,
            |t: &mut $t_class, v| { t$(.$t_prop$([$t_idx])?)+.clone_from(v); }
        )
    };

    // bind target by setter
    // bind!(-> player, Health.color:a:set_a)
    (
        => $t_entity:expr, $t_class:ident$(.$t_prop:ident)+:$t_getter:ident:$t_setter:ident
    ) => {
        $crate::bind::BindTo::new(
            $t_entity.clone(),
            |t: &$t_class, v| &t$(.$t_prop)+.$t_getter() == v,
            |t: &mut $t_class, v| { t$(.$t_prop)+.$t_setter(*v); }
        )
    };
}

#[macro_export]
macro_rules! connect {
    ($entity:expr, |$ctx:ident, $arg:ident: $typ:ty| $cb:expr) => {
        $crate::bind::ConnectionTo::component(
            $entity,
            |$ctx, $arg: &mut ::bevy::prelude::Mut<$typ>| $cb,
        )
    };
    ($entity:expr, |$ctx:ident, $arg:ident: $typ:ty| $cb:block) => {
        $crate::bind::ConnectionTo::component(
            $entity,
            |$ctx, $arg: &mut ::bevy::prelude::Mut<$typ>| $cb,
        )
    };
    ($entity:expr, |$arg:ident: $typ:ty| $cb:expr) => {
        $crate::bind::ConnectionTo::component(
            $entity,
            |_, $arg: &mut ::bevy::prelude::Mut<$typ>| $cb,
        )
    };
    ($entity:expr, |$arg:ident: $typ:ty| $cb:block) => {
        $crate::bind::ConnectionTo::component($entity, |_, $arg| $cb)
    };
    ($entity:expr, |$ctx:ident| $cb:expr) => {
        $crate::bind::ConnectionTo::entity($entity, |$ctx| $cb)
    };
    (|$ctx:ident| $cb:expr) => {
        $crate::bind::ConnectionTo::general(|$ctx| $cb)
    }; // ($entity: expr => $cb:expr) => {
       //     {
       //         let __cb: fn(&mut $crate::bind::EntitySignalContext) = $cb;
       //         $crate::bind::ConnectionTo::entity($entity, __cb)
       //     }
       // };
       // (=> $cb:expr) => {
       //     {
       //         let __cb: fn(&mut $crate::bind::SignalContext) = $cb;
       //         $crate::bind::ConnectionTo::general(__cb)
       //     }
       // };
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Component, Default)]
    struct Health {
        max: f32,
        current: f32,
    }

    #[derive(Component, Default)]
    struct HealthBar {
        value: f32,
        max: f32,
    }

    #[test]
    fn single_property() {
        let mut app = App::new();
        app.add_plugin(BindPlugin);

        let player = app.world.spawn_empty().id();
        let bar = app.world.spawn_empty().id();

        app.world.entity_mut(player).insert(Health::default());
        app.world.entity_mut(bar).insert(HealthBar::default());
        bind!(player, Health.current => bar, HealthBar.value).write(&mut app.world);
        app.update();
        app.update();

        let expected_health = 20.;
        app.world
            .entity_mut(player.clone())
            .get_mut::<Health>()
            .unwrap()
            .current = expected_health;
        app.update();
        let current_health = app
            .world
            .entity(bar.clone())
            .get::<HealthBar>()
            .unwrap()
            .value;
        assert_eq!(
            current_health, expected_health,
            "Bound values should be equals after single update"
        );

        app.update();
        app.update();
        let current_health = app
            .world
            .entity(bar.clone())
            .get::<HealthBar>()
            .unwrap()
            .value;
        assert_eq!(
            current_health, expected_health,
            "Bound values still should be equals after single update"
        );

        let expected_health = 30.;
        app.world
            .entity_mut(player.clone())
            .get_mut::<Health>()
            .unwrap()
            .current = expected_health;
        app.update();
        app.update();
        app.update();
        let current_health = app
            .world
            .entity(bar.clone())
            .get::<HealthBar>()
            .unwrap()
            .value;
        assert_eq!(
            current_health, expected_health,
            "Bound values still should be equals after mutliple updates"
        );
    }

    #[test]
    fn self_bind() {
        let mut app = App::new();
        app.add_plugin(BindPlugin);

        let player = app.world.spawn_empty().id();

        app.world.entity_mut(player).insert(Health::default());

        bind!(player, Health.max => player, Health.current).write(&mut app.world);
        let expected_health = 20.;
        app.world
            .entity_mut(player.clone())
            .get_mut::<Health>()
            .unwrap()
            .max = expected_health;
        app.update();
        let current_health = app
            .world
            .entity(player.clone())
            .get::<Health>()
            .unwrap()
            .current;
        assert_eq!(
            current_health, expected_health,
            "Bound values should be equals after single update"
        );
    }

    #[test]
    fn chain_bind() {
        let mut app = App::new();
        app.add_plugin(BindPlugin);

        let player = app.world.spawn_empty().id();
        let bar = app.world.spawn_empty().id();

        app.world.entity_mut(player).insert(Health::default());
        app.world.entity_mut(bar).insert(HealthBar::default());
        bind!(player, Health.current => bar, HealthBar.value).write(&mut app.world);
        bind!(player, Health.max => player, Health.current).write(&mut app.world);

        let expected_health = 20.;
        app.world
            .entity_mut(player.clone())
            .get_mut::<Health>()
            .unwrap()
            .max = expected_health;
        app.update();
        let visible_health = app
            .world
            .entity(bar.clone())
            .get::<HealthBar>()
            .unwrap()
            .value;
        assert_eq!(
            visible_health, expected_health,
            "Chained values should be equals after single update"
        );
    }
}
