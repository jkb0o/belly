use std::{any::type_name, convert::Infallible, fmt::Debug, marker::PhantomData};

use bevy::{prelude::*, utils::HashMap};
use itertools::Itertools;
use smallvec::SmallVec;
use tagstr::Tag;

use super::RelationsSystems;

pub trait BindableSource: Clone + Send + Sync + 'static {}
impl<T: Clone + Send + Sync + 'static> BindableSource for T {}
pub trait BindableTarget: PartialEq + Clone + Send + Sync + 'static {}
impl<T: PartialEq + Clone + Send + Sync + 'static> BindableTarget for T {}

pub fn process_binds<R: Component, W: Component, S: BindableSource, T: BindableTarget>(
    mut binds: ParamSet<(
        Query<(&Read<R, S>, &R), Changed<R>>,
        Query<(&Write<W, S, T>, &mut W, &mut Change<W>)>,
    )>,
    mut changes: Local<ActiveChanges<S>>,
) {
    changes.clear();
    for (readers, component) in binds.p0().iter() {
        for descriptor in readers.iter() {
            let value = (descriptor.reader)(component).clone();
            changes.add_change(descriptor.target, value, descriptor.id);
        }
    }
    let mut writes = binds.p1();
    for (target, sources) in changes.iter() {
        let Ok((writers, mut component, mut component_change)) = writes.get_mut(*target) else {
            continue
        };
        for (source_value, id) in sources {
            for writer in writers.iter().filter(|w| &w.id == id) {
                let current_value = writer.read(&component);
                match writer.transform(source_value, current_value) {
                    TransformationResult::Changed(new_value) => {
                        writer.write(&mut component, new_value);
                        component_change.report_changed();
                    }
                    TransformationResult::Unchanged => {
                        // Do nothing as nothing got changed
                    }
                    TransformationResult::Invalid(msg) => {
                        error!("Can't transform value for binding: {}", msg);
                        continue;
                    }
                };
            }
        }
    }
}

pub(crate) fn watch_changes<W: Component>(
    something_changed: Query<(), Changed<Change<W>>>,
    mut changes: ResMut<ChangesState>,
) {
    if !something_changed.is_empty() {
        changes.report_changed()
    }
}

#[derive(Deref, DerefMut)]
pub struct ActiveChanges<S: BindableSource>(HashMap<Entity, SmallVec<[(S, BindId); 16]>>);

impl<S: BindableSource> ActiveChanges<S> {
    fn add_change(&mut self, target: Entity, value: S, id: BindId) {
        self.entry(target).or_default().push((value, id));
    }
}
impl<S: BindableSource> Default for ActiveChanges<S> {
    fn default() -> Self {
        ActiveChanges(HashMap::default())
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct BindId {
    source: Tag,
    target: Tag,
}

impl BindId {
    fn new(source: Tag, target: Tag) -> BindId {
        BindId { source, target }
    }
}

#[derive(Resource, Default)]
pub struct ChangesState(usize);
impl ChangesState {
    fn report_changed(&mut self) {
        self.0 += 1;
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

#[derive(Component)]
pub struct Change<W: Component>(PhantomData<W>);
impl<W: Component> Change<W> {
    fn new() -> Change<W> {
        Change(PhantomData)
    }
    fn report_changed(&mut self) {}
}

pub struct ReadDescriptor<R: Component, S: BindableSource> {
    id: BindId,
    target: Entity,
    reader: fn(&R) -> S,
}

impl<R: Component, S: BindableSource> Debug for ReadDescriptor<R, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RaadDescriptor( {} -> {} on {:?} )",
            self.id.source, self.id.target, self.target
        )
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Read<R: Component, S: BindableSource>(Vec<ReadDescriptor<R, S>>);

pub struct WriteDescriptor<W: Component, S: BindableSource, T: BindableTarget> {
    id: BindId,
    transformer: fn(&S, &T) -> TransformationResult<T>,
    reader: for<'a> fn(&'a W) -> &'a T,
    writer: fn(&mut W, T),
}

impl<W: Component, S: BindableSource, T: BindableTarget> WriteDescriptor<W, S, T> {
    fn transform(&self, source: &S, target: &T) -> TransformationResult<T> {
        (self.transformer)(source, target)
    }
    fn read<'a, 'b>(&'a self, component: &'b W) -> &'b T {
        (self.reader)(component)
    }
    fn write(&self, component: &mut W, data: T) {
        (self.writer)(component, data)
    }
}
#[derive(Component, Deref, DerefMut, Default)]
pub struct Write<W: Component, S: BindableSource, T: BindableTarget>(Vec<WriteDescriptor<W, S, T>>);

pub struct BindFrom<R: Component, S: BindableSource> {
    pub source_id: Tag,
    pub source: Entity,
    pub reader: fn(&R) -> S,
}

impl<R: Component, S: BindableSource> BindFrom<R, S> {
    pub fn to<W: Component, T: BindableTarget>(self, to: BindTo<W, S, T>) -> Bind<R, W, S, T> {
        Bind { from: self, to }
    }
}

pub struct BindTo<W: Component, S: BindableSource, T: BindableTarget> {
    pub target: Entity,
    pub target_id: Tag,
    pub transformer: fn(&S, &T) -> TransformationResult<T>,
    pub reader: for<'a> fn(&'a W) -> &'a T,
    pub writer: fn(&mut W, T),
}

impl<W: Component, S: BindableSource, T: BindableTarget> BindTo<W, S, T> {
    pub fn from<R: Component>(self, from: BindFrom<R, S>) -> Bind<R, W, S, T> {
        Bind { from, to: self }
    }
}

pub struct Bind<R: Component, W: Component, S: BindableSource, T: BindableTarget> {
    from: BindFrom<R, S>,
    to: BindTo<W, S, T>,
}

impl<R: Component, W: Component, S: BindableSource, T: BindableTarget> std::fmt::Display
    for Bind<R, W, S, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source_str = self.from.source_id;
        let target_str = self.to.target_id;
        write!(f, "Bind( {source_str} >> {target_str} )")
    }
}

impl<R: Component, W: Component, S: BindableSource, T: BindableTarget> Bind<R, W, S, T> {
    pub fn write(self, world: &mut World) {
        let id = BindId::new(self.from.source_id, self.to.target_id);
        {
            let systems_ref = world.get_resource_or_insert_with(RelationsSystems::default);
            let mut systems = systems_ref.0.write().unwrap();
            systems.add_bind_system::<R, W, S, T>();
        }
        let mut source_entity = world.entity_mut(self.from.source);
        let read_descriptor = ReadDescriptor {
            id,
            target: self.to.target,
            reader: self.from.reader,
        };
        if let Some(mut source_component) = source_entity.get_mut::<Read<R, S>>() {
            source_component.push(read_descriptor);
        } else {
            source_entity.insert(Read(vec![read_descriptor]));
        }

        let mut target_entity = world.entity_mut(self.to.target);
        let write_descriptor = WriteDescriptor {
            id,
            reader: self.to.reader,
            writer: self.to.writer,
            transformer: self.to.transformer,
        };
        if !target_entity.contains::<Change<W>>() {
            target_entity.insert(Change::<W>::new());
        }
        if let Some(mut writer_component) = target_entity.get_mut::<Write<W, S, T>>() {
            writer_component.push(write_descriptor);
        } else {
            target_entity.insert(Write(vec![write_descriptor]));
        }
    }
}

pub enum BindDescriptor<R: Component, W: Component, S: BindableSource, T: BindableTarget> {
    From(BindFrom<R, S>, fn(&S, &T) -> TransformationResult<T>),
    To(BindTo<W, S, T>),
}

impl<R: Component, W: Component, S: BindableSource, T: BindableTarget> std::ops::Shl
    for BindDescriptor<R, W, S, T>
{
    type Output = Bind<R, W, S, T>;

    fn shl(self, rhs: Self) -> Self::Output {
        let (left, right) = (self, rhs);
        match (left, right) {
            // TODO: handle transformer priority:
            // - custom on BindDescriptor::To
            // - custom on Bond::From
            // - default on BindDescriptor::To
            // - default on BindDescriptor::From
            (BindDescriptor::To(to), BindDescriptor::From(from, transformer)) => to.from(from),
            _ => panic!("Invalid binding << operator usage, only to!(...) << from!(...) supprted."),
        }
    }
}

impl<R: Component, W: Component, S: BindableSource, T: BindableTarget> std::ops::Shr
    for BindDescriptor<R, W, S, T>
{
    type Output = Bind<R, W, S, T>;

    fn shr(self, rhs: Self) -> Self::Output {
        let (left, right) = (rhs, self);
        match (left, right) {
            (BindDescriptor::To(to), BindDescriptor::From(from, transformer)) => to.from(from),
            _ => panic!("Invalid binding >> operator usage, only from!(...) >> to!(...) supprted."),
        }
    }
}

pub enum TransformationResult<T: BindableTarget> {
    Changed(T),
    Invalid(String),
    Unchanged,
}

impl<T: BindableTarget> TransformationResult<T> {
    pub fn invalid<S: AsRef<str>>(message: S) -> TransformationResult<T> {
        TransformationResult::Invalid(message.as_ref().to_string())
    }

    pub fn from_error(error: TransformationError) -> TransformationResult<T> {
        TransformationResult::Invalid(error.0)
    }
}

pub struct TransformationError(String);

impl From<Infallible> for TransformationError {
    fn from(_: Infallible) -> Self {
        TransformationError("Unexpected Infallible error. This should never happen".to_string())
    }
}

impl From<String> for TransformationError {
    fn from(s: String) -> Self {
        TransformationError(s)
    }
}

pub fn transform<
    T: TryFrom<F, Error = E> + BindableTarget,
    F: Clone,
    E: Into<TransformationError>,
>(
    incoming: &F,
    current: &T,
) -> TransformationResult<T> {
    let new_value = match T::try_from(incoming.clone()) {
        Err(err) => return TransformationResult::from_error(err.into()),
        Ok(val) => val,
    };
    if &new_value != current {
        return TransformationResult::Changed(new_value);
    } else {
        return TransformationResult::Unchanged;
    }
}

pub fn format_source_id<T>(field: &str) -> String {
    format!(
        "{}:{}",
        type_name::<T>().split_whitespace().join(""),
        field.trim().trim_matches('.').split_whitespace().join("")
    )
}

#[macro_export]
macro_rules! bind {
    (@bind component to $entity:expr, $cls:ty, { $($prop:tt)+ }, $filter:expr) => {
        $crate::relations::bind::BindDescriptor::To($crate::relations::bind::BindTo {
            target_id: ::tagstr::tag!($crate::relations::bind::format_source_id::<$cls>(stringify!($($prop)+))),
            reader: |c: &$cls| &c.$($prop)+,
            transformer: $filter,
            writer: |c: &mut $cls, v| c.$($prop)+ = v,
            target: $entity
        })
    };
    (@bind component from $entity:expr, $cls:ty, { $($prop:tt)+ }, $filter:expr) => {
        $crate::relations::bind::BindDescriptor::From($crate::relations::bind::BindFrom {
            source_id: ::tagstr::tag!($crate::relations::bind::format_source_id::<$cls>(stringify!($($prop)+))),
            source: $entity,
            reader: |c: &$cls| c.$($prop)+.clone()
        }, $filter)
    };

    (@filter fmt:$val:ident( $($fmt:tt)* ) ) => {
        |s, t| {
            let $val = s;
            let $val = format!($($fmt)*);
            if &$val == t {
                $crate::relations::bind::TransformationResult::Unchanged
            } else {
                $crate::relations::bind::TransformationResult::Changed($val)
            }
        }
    };
    (@filter $converter:ident:$method:ident ) => {
        |s, t| {
            $crate::Transformers::$converter().$method(s, t)
        }
    };
    (@filter default) => {
        |s, t| {
            $crate::relations::bind::transform(s, t)
        }
    };

    // only filters here, can bind actually
    (@args {$mode:ident, $direction:ident, $entity:expr, $cls:ty}, $prop:tt | $($filter:tt)+ ) => {
        $crate::bind!(@bind $mode $direction $entity, $cls, $prop, $crate::bind!(@filter $($filter)+))
    };

    (@args {$mode:ident, $direction:ident,  $entity:expr, $cls:ty}, $prop:tt) => {
        $crate::bind!(@bind $mode $direction $entity, $cls, $prop, $crate::bind!(@filter default))
    };

    // adding the rest of props, everyting before |
    (@args $h:tt, {$($props:tt)+} [$($idx:tt)+] $($rest:tt)*) => {
        $crate::bind!(@args $h, {$($props)+[$($idx)+]} $($rest)*)
    };

    (@args $h:tt, {$($props:tt)+} ($($call:tt)*) $($rest:tt)*) => {
        $crate::bind!(@args $h, {$($props)+($($call)*)} $($rest)*)
    };

    (@args $h:tt, {$($props:tt)+} $part:tt $($rest:tt)*) => {
        $crate::bind!(@args $h, {$($props)+$part} $($rest)*)
    };

    // (@args $h:tt, {$($props:tt)+} $part:literal $($rest:tt)*) => {
    //     $crate::bind!(@args $h, {$($props)+$part} $($rest)*)
    // };


    (@args $h:tt, {$($props:tt)+} . $($rest:tt)*) => {
        $crate::bind!(@args $h, {$($props)+.} $($rest)*)
    };


    //sinle part should be handled separatly
    // add indexed field
    // (@args $h:tt: $first:tt[$($idx:tt)+] $($args:tt)*) => {
    //     $crate::bind!(@args $h, {$first[$($idx)+]} $($args)* )
    // };
    // // add method call
    // (@args $h:tt: $first:tt($($call:tt)*) $($args:tt)*) => {
    //     $crate::bind!(@args $h, {$first($($call)*)} $($args)* )
    // };
    // add field
    (@args $h:tt: $first:ident $($args:tt)*) => {
        $crate::bind!(@args $h, {$first} $($args)* )
    };
    (@args $h:tt: $first:tt $($args:tt)*) => {
        $crate::bind!(@args $h, {$first} $($args)* )
    };

    // start here and move up
    ( $direction:ident, $entity:expr, $cls:ty: $($args:tt)+ ) => {
        $crate::bind!(@args {component, $direction, $entity, $cls}: $($args)+ )
    };
    ( $direction:ident, $cls:ty: $($args:tt)+ ) => {
        $crate::bind!(@args {resource, $direction, $entity, $cls}: $($args)+ )
    };
}

#[macro_export]
macro_rules! from {
    ( $($bind:tt)* ) => { $crate::bind!(from, $($bind)*) };
}

#[macro_export]
macro_rules! to {
    ( $($bind:tt)* ) => { $crate::bind!(to, $($bind)*) };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    #[derive(Component, Default)]
    struct Health {
        max: f32,
        current: f32,
    }

    #[derive(Component, Default)]
    struct HealthBar {
        value: f32,
        _max: f32,
    }

    #[test]
    fn single_property() {
        let mut app = App::new();
        app.add_plugin(RelationsPlugin);

        let player = app.world.spawn_empty().id();
        let bar = app.world.spawn_empty().id();

        app.world.entity_mut(player).insert(Health::default());
        app.world.entity_mut(bar).insert(HealthBar::default());
        let bind = from!(player, Health: current) >> to!(bar, HealthBar: value);
        bind.write(&mut app.world);
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
            "Bind values should be equals after single update"
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
            "Bind values still should be equals after single update"
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
            "Bind values still should be equals after mutliple updates"
        );
    }

    #[test]
    fn chain_bind() {
        let mut app = App::new();
        app.add_plugin(RelationsPlugin);

        let player = app.world.spawn_empty().id();
        let bar = app.world.spawn_empty().id();

        app.world.entity_mut(player).insert(Health::default());
        app.world.entity_mut(bar).insert(HealthBar::default());
        let bind = from!(player, Health: current) >> to!(bar, HealthBar: value);
        bind.write(&mut app.world);
        let bind = from!(player, Health: max) >> to!(player, Health: current);
        bind.write(&mut app.world);
        app.update();

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
