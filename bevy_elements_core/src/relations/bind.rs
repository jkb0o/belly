use super::RelationsSystems;
use bevy::{ecs::system::Command, prelude::*, utils::HashMap};
use std::{
    any::{type_name, TypeId},
    marker::PhantomData,
};

pub trait BindValue: Default + PartialEq + Clone + Send + Sync + 'static {}
impl<T: Default + PartialEq + Clone + Send + Sync + 'static> BindValue for T {}

#[derive(Component, Deref, DerefMut)]
pub struct BindingChanges<T: BindValue>(HashMap<usize, T>);

#[derive(Resource, Default)]
pub struct ChangeCounter(usize);

impl ChangeCounter {
    pub fn add(&mut self) {
        self.0 += 1;
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

impl<T: BindValue> BindingChanges<T> {
    pub fn new() -> BindingChanges<T> {
        BindingChanges(default())
    }
}

#[derive(Component, Deref)]
pub struct BindingSource<R: Component, T: BindValue>(Vec<(Entity, fn(&R) -> T, usize)>);

impl<R: Component, T: BindValue> BindingSource<R, T> {
    pub fn new() -> BindingSource<R, T> {
        BindingSource(default())
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct BindingTarget<W: Component, T: BindValue>(
    HashMap<usize, (fn(&W, &T) -> bool, fn(&mut W, &T))>,
);

impl<W: Component, T: BindValue> BindingTarget<W, T> {
    fn new() -> BindingTarget<W, T> {
        BindingTarget(default())
    }
}

#[derive(Resource)]
pub struct Changes<T: Component>(PhantomData<T>);
impl<T: Component> Default for Changes<T> {
    fn default() -> Self {
        Changes::<T>(PhantomData::<T>)
    }
}

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
            let systems = world.get_resource_or_insert_with(RelationsSystems::default);
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
            let systems_ref = world.get_resource_or_insert_with(RelationsSystems::default);
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
        $crate::relations::Bind::build(
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
        $crate::relations::Bind::build(
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
        $crate::relations::Bind::build(
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
        $crate::relations::Bind::build(
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
        $crate::relations::BindFrom::new(
            $s_entity.clone(),
            |s: &$s_class| { s$(.$s_prop)+.clone() },
        )
    };
    (
        <= $s_entity:expr, $s_class:ident$(.$s_prop:ident)+
    ) => {
        $crate::relations::BindFrom::new(
            $s_entity.clone(),
            |s: &$s_class| { s$(.$s_prop)+.clone() },
        )
    };
    // bind source by getter
    // bind!(player, Health.color:a ->)
    (
        $s_entity:expr, $s_class:ident$(.$s_prop:ident)+:$s_getter:ident =>
    ) => {
        $crate::relations::BindFrom::new(
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
        $crate::relations::BindTo::new(
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
        $crate::relations::BindTo::new(
            $t_entity.clone(),
            |t: &$t_class, v| &t$(.$t_prop)+.$t_getter() == v,
            |t: &mut $t_class, v| { t$(.$t_prop)+.$t_setter(*v); }
        )
    };
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
