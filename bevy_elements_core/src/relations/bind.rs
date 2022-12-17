use std::{
    any::type_name,
    convert::Infallible,
    fmt::Debug,
    marker::PhantomData,
    num::ParseFloatError,
    ops::{Deref, DerefMut},
};

use bevy::{ecs::system::Command, prelude::*, utils::HashMap};
use itertools::Itertools;
use smallvec::SmallVec;
use tagstr::Tag;

use super::RelationsSystems;

pub type SourceReader<R, S> = fn(&R) -> S;
pub type Transformer<S, T> = fn(&S, Prop<T>) -> TransformationResult;
pub type RefReader<W, T> = for<'b> fn(&'b Mut<W>) -> &'b T;
pub type MutReader<W, T> = for<'b> fn(&'b mut Mut<W>) -> &'b mut T;
pub type TransformationResult = Result<(), TransformationError>;

pub trait BindableSource: Clone + Send + Sync + 'static {}
impl<T: Clone + Send + Sync + 'static> BindableSource for T {}
pub trait BindableTarget: PartialEq + Send + Sync + 'static {}
impl<T: PartialEq + Send + Sync + 'static> BindableTarget for T {}

fn write_component_changes<W: Component, S: BindableSource, T: BindableTarget>(
    changes: &ActiveChanges<S>,
    writes: &mut Query<(&WriteComponent<W, S, T>, &mut W, &mut Change<W>)>,
) {
    for (target, sources) in changes.iter() {
        let Ok((writers, mut component, mut component_change)) = writes.get_mut(*target) else {
            continue
        };
        for (source, id) in sources {
            for write_descriptor in writers.iter().filter(|w| &w.id == id) {
                let mut prop_descriptor = write_descriptor.prop_descripror(&mut component);
                if let Err(e) = write_descriptor.transform(source, prop_descriptor.prop()) {
                    error!("Error transforming {:?}: {}", id, e.0);
                } else if prop_descriptor.changed {
                    // TODO: protect infinity circular loops by tracking property changes
                    component_change.set_changed();
                }
            }
        }
    }
}

pub fn component_to_component_system<
    R: Component,
    W: Component,
    S: BindableSource,
    T: BindableTarget,
>(
    mut binds: ParamSet<(
        Query<(&ReadComponent<R, S>, &R), Changed<R>>,
        Query<(&WriteComponent<W, S, T>, &mut W, &mut Change<W>)>,
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
    write_component_changes(&mut changes, &mut writes);
}

pub fn resource_to_component_system<
    R: Resource,
    W: Component,
    S: BindableSource,
    T: BindableTarget,
>(
    res: Res<R>,
    read: Res<ReadResource<R, S>>,
    mut writes: Query<(&WriteComponent<W, S, T>, &mut W, &mut Change<W>)>,
    mut changes: Local<ActiveChanges<S>>,
) {
    if !res.is_changed() {
        return;
    }
    changes.clear();

    for descriptor in read.iter() {
        let value = (descriptor.reader)(&res);
        changes.add_change(descriptor.target, value, descriptor.id);
    }
    write_component_changes(&mut changes, &mut writes);
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

pub struct PropertyDescriptor<'a, 'c, C: Component, T> {
    changed: bool,
    component: &'a mut Mut<'c, C>,
    ref_getter: for<'b> fn(&'b Mut<C>) -> &'b T,
    mut_getter: for<'b> fn(&'b mut Mut<C>) -> &'b mut T,
}

impl<'a, 'c, C: Component, T> PropertyDescriptor<'a, 'c, C, T> {
    fn prop(&mut self) -> Prop<T> {
        Prop(self)
    }
}

impl<'a, 'c, C: Component, T> AsRef<T> for PropertyDescriptor<'a, 'c, C, T> {
    fn as_ref(&self) -> &T {
        (self.ref_getter)(&self.component)
    }
}

impl<'a, 'c, C: Component, T> AsMut<T> for PropertyDescriptor<'a, 'c, C, T> {
    fn as_mut(&mut self) -> &mut T {
        self.changed = true;
        (self.mut_getter)(&mut self.component)
    }
}

pub trait PropertyProtocol<T>: AsRef<T> + AsMut<T> {}
impl<T, X: AsRef<T> + AsMut<T>> PropertyProtocol<T> for X {}

pub struct Prop<'a, T>(&'a mut dyn PropertyProtocol<T>);

impl<'a, T> Deref for Prop<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<'a, T> DerefMut for Prop<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

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
}

pub struct ReadDescriptor<R, S: BindableSource> {
    id: BindId,
    target: Entity,
    reader: SourceReader<R, S>,
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
pub struct ReadComponent<R: Component, S: BindableSource>(Vec<ReadDescriptor<R, S>>);

#[derive(Resource, Deref, DerefMut)]
pub struct ReadResource<R: Resource, S: BindableSource>(Vec<ReadDescriptor<R, S>>);

impl<R: Resource, S: BindableSource> Default for ReadResource<R, S> {
    fn default() -> Self {
        ReadResource(vec![])
    }
}

pub struct WriteDescriptor<W, S: BindableSource, T: BindableTarget> {
    id: BindId,
    transformer: Transformer<S, T>,
    ref_getter: RefReader<W, T>,
    mut_getter: MutReader<W, T>,
}

impl<W: Component, S: BindableSource, T: BindableTarget> WriteDescriptor<W, S, T> {
    fn prop_descripror<'a, 'c>(
        &self,
        component: &'a mut Mut<'c, W>,
    ) -> PropertyDescriptor<'a, 'c, W, T> {
        PropertyDescriptor {
            component,
            changed: false,
            ref_getter: self.ref_getter,
            mut_getter: self.mut_getter,
        }
    }

    fn transform(&self, source: &S, prop: Prop<T>) -> TransformationResult {
        (self.transformer)(source, prop)
    }
}
#[derive(Component, Deref, DerefMut, Default)]
pub struct WriteComponent<W: Component, S: BindableSource, T: BindableTarget>(
    Vec<WriteDescriptor<W, S, T>>,
);

pub struct FromComponent<R: Component, S: BindableSource> {
    pub id: Tag,
    pub source: Entity,
    pub reader: SourceReader<R, S>,
}

impl<R: Component, S: BindableSource> FromComponent<R, S> {
    pub fn bind_component<W: Component, T: BindableTarget>(
        self,
        to: ToComponent<W, S, T>,
    ) -> ComponentToComponent<R, W, S, T> {
        ComponentToComponent { from: self, to }
    }
}

pub struct FromComponentWithTransformer<R: Component, S: BindableSource, T: BindableTarget> {
    pub from: FromComponent<R, S>,
    pub transformer: Transformer<S, T>,
}

impl<R: Component, S: BindableSource, T: BindableTarget> FromComponentWithTransformer<R, S, T> {
    pub fn bind<W: Component>(
        self,
        to: ToComponentWithoutTransformer<W, T>,
    ) -> ComponentToComponent<R, W, S, T> {
        let transformer = self.transformer;
        self.from.bind_component(ToComponent {
            id: to.id,
            target: to.target,
            writer: to.writer,
            reader: to.reader,
            transformer,
        })
    }
}

pub struct FromResource<R: Resource, S: BindableSource> {
    pub id: Tag,
    pub reader: SourceReader<R, S>,
}

impl<R: Resource, S: BindableSource> FromResource<R, S> {
    pub fn bind_component<W: Component, T: BindableTarget>(
        self,
        to: ToComponent<W, S, T>,
    ) -> ResourceToComponent<R, W, S, T> {
        ResourceToComponent { from: self, to }
    }
}

pub struct FromResourceWithTransformer<R: Resource, S: BindableSource, T: BindableTarget> {
    pub from: FromResource<R, S>,
    pub transformer: Transformer<S, T>,
}
impl<R: Resource, S: BindableSource, T: BindableTarget> FromResourceWithTransformer<R, S, T> {
    pub fn bind_component<W: Component>(
        self,
        to: ToComponentWithoutTransformer<W, T>,
    ) -> ResourceToComponent<R, W, S, T> {
        ResourceToComponent {
            from: self.from,
            to: ToComponent {
                id: to.id,
                target: to.target,
                writer: to.writer,
                reader: to.reader,
                transformer: self.transformer,
            },
        }
    }
}

// pub struct ToCmp<W, S, T>
// where
//     W: Component,
//     S: BindableSource,
//     T: BindableTarget,
// {
//     pub id: Tag,
//     pub target: Entity,
//     pub transformer: Box<dyn Fn(&S, &T) -> TransformationResult<T>>,
//     pub reader: TargetReader<W, T>,
//     pub writer: fn(&mut W, T),
// }

pub struct ToComponent<W: Component, S: BindableSource, T: BindableTarget> {
    pub id: Tag,
    pub target: Entity,
    pub transformer: Transformer<S, T>,
    pub reader: RefReader<W, T>,
    pub writer: MutReader<W, T>,
}

impl<W: Component, S: BindableSource, T: BindableTarget> ToComponent<W, S, T> {
    pub fn bind_component<R: Component>(
        self,
        from: FromComponent<R, S>,
    ) -> ComponentToComponent<R, W, S, T> {
        ComponentToComponent { from, to: self }
    }
    pub fn bind_resource<R: Resource>(
        self,
        from: FromResource<R, S>,
    ) -> ResourceToComponent<R, W, S, T> {
        ResourceToComponent { from, to: self }
    }
}

pub struct ToComponentWithoutTransformer<W: Component, T: BindableTarget> {
    pub id: Tag,
    pub target: Entity,
    pub reader: RefReader<W, T>,
    pub writer: MutReader<W, T>,
}

impl<W: Component, T: BindableTarget> ToComponentWithoutTransformer<W, T> {
    pub fn bind_component<R: Component, S: BindableSource>(
        self,
        from: FromComponentWithTransformer<R, S, T>,
    ) -> ComponentToComponent<R, W, S, T> {
        from.from.bind_component(ToComponent {
            id: self.id,
            target: self.target,
            reader: self.reader,
            writer: self.writer,
            transformer: from.transformer,
        })
    }
    pub fn bind_resource<R: Resource, S: BindableSource>(
        self,
        from: FromResourceWithTransformer<R, S, T>,
    ) -> ResourceToComponent<R, W, S, T> {
        from.from.bind_component(ToComponent {
            id: self.id,
            target: self.target,
            reader: self.reader,
            writer: self.writer,
            transformer: from.transformer,
        })
    }
    // pub fn with_transormer<S: BindableSource>(self, transformator: fn())
}

pub trait Transformable {
    type Transformer;
    fn transformer() -> Self::Transformer;
}

pub struct ToComponentTransformable<W: Component, T: BindableTarget + Transformable> {
    pub id: Tag,
    pub target: Entity,
    pub reader: RefReader<W, T>,
    pub writer: MutReader<W, T>,
}

impl<W: Component, T: BindableTarget + Transformable> ToComponentTransformable<W, T> {
    pub fn transformed<S: BindableSource>(
        self,
        make_transformer: fn(T::Transformer) -> Transformer<S, T>,
    ) -> ToComponent<W, S, T> {
        ToComponent {
            id: self.id,
            target: self.target,
            reader: self.reader,
            writer: self.writer,
            transformer: make_transformer(T::transformer()),
        }
    }
}

fn register_component_writer<W: Component, S: BindableSource, T: BindableTarget>(
    world: &mut World,
    id: BindId,
    to: ToComponent<W, S, T>,
) {
    let mut target_entity = world.entity_mut(to.target);
    let write_descriptor = WriteDescriptor {
        id,
        ref_getter: to.reader,
        mut_getter: to.writer,
        transformer: to.transformer,
    };
    if !target_entity.contains::<Change<W>>() {
        target_entity.insert(Change::<W>::new());
    }
    if let Some(mut writer_component) = target_entity.get_mut::<WriteComponent<W, S, T>>() {
        writer_component.push(write_descriptor);
    } else {
        target_entity.insert(WriteComponent(vec![write_descriptor]));
    }
}

pub struct ComponentToComponent<R: Component, W: Component, S: BindableSource, T: BindableTarget> {
    from: FromComponent<R, S>,
    to: ToComponent<W, S, T>,
}

impl<R: Component, W: Component, S: BindableSource, T: BindableTarget> std::fmt::Display
    for ComponentToComponent<R, W, S, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source_str = self.from.id;
        let target_str = self.to.id;
        write!(f, "ComponentToComponent( {source_str} >> {target_str} )")
    }
}

impl<R: Component, W: Component, S: BindableSource, T: BindableTarget> Command
    for ComponentToComponent<R, W, S, T>
{
    fn write(self, world: &mut World) {
        self.write(world);
    }
}

impl<R: Component, W: Component, S: BindableSource, T: BindableTarget>
    ComponentToComponent<R, W, S, T>
{
    pub fn write(self, world: &mut World) {
        {
            let systems_ref = world.get_resource_or_insert_with(RelationsSystems::default);
            let mut systems = systems_ref.0.write().unwrap();
            systems.add_component_to_component::<R, W, S, T>();
        }
        let id = BindId::new(self.from.id, self.to.id);
        let mut source_entity = world.entity_mut(self.from.source);
        let read_descriptor = ReadDescriptor {
            id,
            target: self.to.target,
            reader: self.from.reader,
        };
        if let Some(mut source_component) = source_entity.get_mut::<ReadComponent<R, S>>() {
            source_component.push(read_descriptor);
        } else {
            source_entity.insert(ReadComponent(vec![read_descriptor]));
        }
        register_component_writer(world, id, self.to);
    }
}

pub struct ResourceToComponent<R: Resource, W: Component, S: BindableSource, T: BindableTarget> {
    from: FromResource<R, S>,
    to: ToComponent<W, S, T>,
}

impl<R: Resource, W: Component, S: BindableSource, T: BindableTarget> std::fmt::Display
    for ResourceToComponent<R, W, S, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source_str = self.from.id;
        let target_str = self.to.id;
        write!(f, "ResourceToComponent( {source_str} >> {target_str} )")
    }
}

impl<R: Resource, W: Component, S: BindableSource, T: BindableTarget>
    ResourceToComponent<R, W, S, T>
{
    pub fn write(self, world: &mut World) {
        {
            let systems_ref = world.get_resource_or_insert_with(RelationsSystems::default);
            let mut systems = systems_ref.0.write().unwrap();
            systems.add_resource_to_component::<R, W, S, T>();
        }
        let id = BindId::new(self.from.id, self.to.id);
        let read_descriptor = ReadDescriptor {
            id,
            target: self.to.target,
            reader: self.from.reader,
        };
        world
            .get_resource_or_insert_with(ReadResource::<R, S>::default)
            .push(read_descriptor);
        register_component_writer(world, id, self.to);
    }
}

// pub enum TransformationResult<T: BindableTarget> {
//     Changed(T),
//     Invalid(String),
//     Unchanged,
// }

// impl<T: BindableTarget> TransformationResult<T> {
//     pub fn invalid<S: AsRef<str>>(message: S) -> TransformationResult<T> {
//         TransformationResult::Invalid(message.as_ref().to_string())
//     }

//     pub fn from_error(error: TransformationError) -> TransformationResult<T> {
//         TransformationResult::Invalid(error.0)
//     }
// }

pub struct TransformationError(String);

impl TransformationError {
    pub fn new(value: String) -> TransformationError {
        TransformationError(value)
    }
}

impl From<Infallible> for TransformationError {
    fn from(_: Infallible) -> Self {
        TransformationError("Unexpected Infallible error. This should never happen".to_string())
    }
}

impl From<ParseFloatError> for TransformationError {
    fn from(e: ParseFloatError) -> Self {
        TransformationError(format!("{e}"))
    }
}

impl From<String> for TransformationError {
    fn from(s: String) -> Self {
        TransformationError(s)
    }
}

pub fn bind_id<T>(field: &str) -> Tag {
    Tag::new(format!(
        "{}:{}",
        type_name::<T>().split_whitespace().join(""),
        field.trim().trim_matches('.').split_whitespace().join("")
    ))
}

#[macro_export]
macro_rules! bind {
    // from!(entity, Component:some.property)
    (@bind from component $entity:expr, $cls:ty, { $($prop:tt)+ }, default) => {
        $crate::relations::bind::FromComponent {
            id: $crate::relations::bind::bind_id::<$cls>(stringify!($($prop)+)),
            source: $entity,
            reader: |c: &$cls| c.$($prop)+.clone()
        }
    };
    // from!(Resource:some.property)
    (@bind from resource $cls:ty, { $($prop:tt)+ }, default) => {
        $crate::relations::bind::FromResource {
            id: $crate::relations::bind::bind_id::<$cls>(stringify!($($prop)+)),
            reader: |c: &$cls| c.$($prop)+.clone()
        }
    };
    // from!(entity, Component:some.property | some:transformer)
    (@bind from component $entity:expr, $cls:ty, { $($prop:tt)+ }, $transformer:expr) => {
        $crate::relations::bind::FromComponentWithTransformer {
            transformer: $transformer,
            from: $crate::relations::bind::FromComponent {
                id: $crate::relations::bind::bind_id::<$cls>(stringify!($($prop)+)),
                source: $entity,
                reader: |c: &$cls| c.$($prop)+.clone()
            }
        }
    };
    // from!(Resource:some.property | some:transformer)
    (@bind from resource $cls:ty, { $($prop:tt)+ }, $transformer:expr) => {
        $crate::relations::bind::FromResourceWithTransformer {
            transformer: $transformer,
            from: $crate::relations::bind::FromResource {
                id: $crate::relations::bind::bind_id::<$cls>(stringify!($($prop)+)),
                reader: |c: &$cls| c.$($prop)+.clone()
            }
        }
    };
    // to!(entity, Component:some.property)
    (@bind to component $entity:expr, $cls:ty, { $($prop:tt)+ }, default) => {
        $crate::relations::bind::ToComponentWithoutTransformer {
            id: $crate::relations::bind::bind_id::<$cls>(stringify!($($prop)+)),
            target: $entity,
            reader: |c: &::bevy::prelude::Mut<$cls>| &c.$($prop)+,
            writer: |c: &mut ::bevy::prelude::Mut<$cls>| &mut c.$($prop)+,
        }
    };
    (@bind to component $entity:expr, $cls:ty, { $($prop:tt)+ }, transformable $transformer:ident ) => {
        $crate::relations::bind::ToComponentTransformable {
            id: $crate::relations::bind::bind_id::<$cls>(stringify!($($prop)+)),
            target: $entity,
            reader: |c: &::bevy::prelude::Mut<$cls>| &c.$($prop)+,
            writer: |c: &mut ::bevy::prelude::Mut<$cls>| &mut c.$($prop)+,
        }.transformed(|tr| tr.$transformer())
    };
    // to!(entity, Component:some.propery | some:transformer)
    (@bind to component $entity:expr, $cls:ty, { $($prop:tt)+ }, $transformer:expr) => {
        $crate::relations::bind::ToComponent {
            id: $crate::relations::bind::bind_id::<$cls>(stringify!($($prop)+)),
            target: $entity,
            reader: |c: &::bevy::prelude::Mut<$cls>| &c.$($prop)+,
            writer: |c: &mut ::bevy::prelude::Mut<$cls>| &mut c.$($prop)+,
            transformer: $transformer,
        }
    };


    (@transform fmt:$val:ident( $($fmt:tt)* ) ) => {
        |s, mut t| {
            let $val = s;
            let $val = format!($($fmt)*);
            if $val != *t {
                *t = $val;

            }
            Ok(())
        }
    };
    (@transform $converter:ident:$method:ident ) => {
        |s, t| {
            $crate::Transformers::$converter().$method(s, t)
        }
    };
    (@transform $converter:ident:$method:ident($($args:tt)*) ) => {
        |s, t| {
            $crate::Transformers::$converter().$method(s, t, $($args)*)
        }
    };

    // only transformers here, can bind actually
    (@args {$mode:ident from $entity:expr, $cls:ty}, $prop:tt) => {
        $crate::bind!(@bind from $mode $entity, $cls, $prop, default)
    };
    (@args {$mode:ident from $cls:ty}, $prop:tt) => {
        $crate::bind!(@bind from $mode $cls, $prop, default)
    };

    (@args {$mode:ident to $entity:expr, $cls:ty}, $prop:tt) => {
        $crate::bind!(@bind to $mode $entity, $cls, $prop, default)
    };


    (@args {$mode:ident $direction:ident $cls:ty}, $prop:tt | $($transformer:tt)+ ) => {
        $crate::bind!(@bind $direction $mode $cls, $prop, $crate::bind!(@transform $($transformer)+))
    };

    (@args {$mode:ident $direction:ident $entity:expr, $cls:ty}, $prop:tt | $transformer:ident ) => {
        $crate::bind!(@bind $direction $mode $entity, $cls, $prop, transformable $transformer)
    };
    (@args {$mode:ident $direction:ident $entity:expr, $cls:ty}, $prop:tt | $($transformer:tt)+ ) => {
        $crate::bind!(@bind $direction $mode $entity, $cls, $prop, $crate::bind!(@transform $($transformer)+))
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

    (@args $h:tt, {$($props:tt)+} . $($rest:tt)*) => {
        $crate::bind!(@args $h, {$($props)+.} $($rest)*)
    };


    // add first ident (or tuple index) of field manually
    (@args $h:tt: $first:ident $($args:tt)*) => {
        $crate::bind!(@args $h, {$first} $($args)* )
    };
    (@args $h:tt: $first:tt $($args:tt)*) => {
        $crate::bind!(@args $h, {$first} $($args)* )
    };

    // start here and move up
    ( $direction:ident $cls:ty: $($args:tt)+ ) => {
        $crate::bind!(@args {resource $direction $cls}: $($args)+ )
    };
    ( $direction:ident $entity:expr, $cls:ty: $($args:tt)+ ) => {
        $crate::bind!(@args {component $direction $entity, $cls}: $($args)+ )
    };
    ( => $entity:expr, $cls:ty: $($args:tt)+ ) => {
        $crate::bind!(@args {component to $entity, $cls}: $($args)+ )
    };
    ( <= $entity:expr, $cls:ty: $($args:tt)+ ) => {
        $crate::bind!(@args {component from $entity, $cls}: $($args)+ )
    };
}

#[macro_export]
macro_rules! from {
    ( $($bind:tt)* ) => { $crate::bind!(from $($bind)*) };
}

#[macro_export]
macro_rules! to {
    ( $($bind:tt)* ) => { $crate::bind!(to $($bind)*) };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ColorTransformerExtension;
    use crate::*;

    #[derive(Component, Default)]
    struct Health {
        max: f32,
        current: f32,
    }

    impl Health {
        fn percent(&self) -> f32 {
            self.current / self.max
        }
    }

    #[derive(Component, Default)]
    struct HealthBar {
        value: f32,
        output: String,
        color: Color,
        _max: f32,
    }

    #[derive(Default, Clone, PartialEq)]
    enum BtnMode {
        #[default]
        Press,
        Instant,
        Toggle,
    }

    impl TryFrom<&str> for BtnMode {
        type Error = String;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "press" => Ok(BtnMode::Press),
                "instant" => Ok(BtnMode::Instant),
                "toggle" => Ok(BtnMode::Toggle),
                _ => Err(format!("Can't parse `{}` as BtnMode", value)),
            }
        }
    }

    impl TryFrom<String> for BtnMode {
        type Error = String;
        fn try_from(value: String) -> Result<Self, Self::Error> {
            BtnMode::try_from(value.as_str())
        }
    }

    impl From<BtnMode> for String {
        fn from(mode: BtnMode) -> Self {
            match mode {
                BtnMode::Press => "press",
                BtnMode::Instant => "instant",
                BtnMode::Toggle => "toggle",
            }
            .to_string()
        }
    }

    #[derive(Component)]
    struct Btn {
        mode: BtnMode,
    }
    fn btn_bind_mode_to(target: Entity) -> ToComponentWithoutTransformer<Btn, BtnMode> {
        to!(target, Btn: mode)
    }
    fn btn_bind_from_mode(source: Entity) -> FromComponent<Btn, BtnMode> {
        from!(source, Btn: mode)
    }

    #[test]
    fn test_macro_compiles() {
        // components
        let mut world = World::new();
        let e = world.spawn_empty().id();
        let _bind = from!(e, Health: current) >> to!(e, HealthBar: value);
        let _bind = to!(e, HealthBar: value) << from!(e, Health: current);

        let _bind = from!(e, Health: current) >> to!(e, HealthBar:output | fmt:val("{val}"));
        let _bind = from!(e, Health:current  | fmt:val("{val}")) >> to!(e, HealthBar: output);

        let _bind = from!(e, Health: percent()) >> to!(e, HealthBar: color | color: one_minus_r);
        let _bind = to!(e, HealthBar: color | color: r) << from!(e, Health: percent());
        let _bind = from!(e, Health: percent() | color: r) >> to!(e, HealthBar: color);
        let _bind =
            to!(e, HealthBar: color) << from!(e, Health: percent() | color:lerp_r(0.2, 0.8));

        let _bind = from!(e, HealthBar: output) >> to!(e, Btn: mode);
        let _bind = to!(e, Btn: mode) << from!(e, HealthBar: output);

        let _bind = btn_bind_from_mode(e) >> to!(e, HealthBar: output);
        let _bind = from!(e, HealthBar: output) >> btn_bind_mode_to(e);

        // resources
        let _bind = from!(Time: elapsed_seconds()) >> to!(e, Health: current);
        let _bind = to!(e, Health: current) << from!(Time: elapsed_seconds());
        let _bind = from!(Time:elapsed_seconds() | fmt:val("{val}")) >> to!(e, HealthBar: output);
        let _bind =
            to!(e, HealthBar: output) << from!(Time:elapsed_seconds() | fmt:val("{val:0.3}"));
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
