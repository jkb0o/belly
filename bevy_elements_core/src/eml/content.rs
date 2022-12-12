use bevy::ecs::system::Command;
use bevy::prelude::*;
use std::any::TypeId;

use crate::{relations::*, Element, ElementsBuilder};

pub trait IntoContent {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity>;
}

impl IntoContent for String {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity> {
        let mut entity = world.entity_mut(parent);
        if let Some(mut text) = entity.get_mut::<Text>() {
            text.sections[0].value = self;
        } else {
            let text = Text::from_section(self, Default::default());
            entity
                .insert(Element::inline())
                .insert(TextBundle { text, ..default() });
        }
        vec![parent]
    }
}

#[derive(Component)]
struct BindContent<T: Bindable + IntoContent + std::fmt::Debug> {
    value: T,
}
impl<C: Component, T: Bindable + IntoContent + std::fmt::Debug> IntoContent for BindFrom<C, T> {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity> {
        world
            .entity_mut(parent)
            .insert(NodeBundle::default())
            .insert(BindContent {
                value: T::default(),
            });
        let target = BindTo::new(
            parent,
            |t: &BindContent<T>, v| &t.value == v,
            |t: &mut BindContent<T>, v| t.value.clone_from(v),
        );
        let bind = Bind::new(self, target);
        bind.write(world);
        let systems_ref = world.get_resource_or_insert_with(RelationsSystems::default);
        let mut systems = systems_ref.0.write().unwrap();
        systems.add_custom_system(TypeId::of::<BindContent<T>>(), bind_component_system::<T>);

        vec![parent]
    }
}

fn bind_component_system<T: Bindable + IntoContent + std::fmt::Debug>(
    mut commands: Commands,
    binds: Query<(Entity, &BindContent<T>), Changed<BindContent<T>>>,
) {
    // info!("bindsystem for {}", type_name::<T>());
    for (entity, bind) in binds.iter() {
        let content = bind.value.clone();
        info!("bind value changed for {:?}", entity);
        commands.add(move |world: &mut World| {
            info!("setting value: bind value changed to {:?}", content);
            content.into_content(entity, world);
        })
    }
}

impl IntoContent for Vec<Entity> {
    fn into_content(self, _parent: Entity, _world: &mut World) -> Vec<Entity> {
        self
    }
}

impl<T: Iterator, F: Fn(T::Item) -> ElementsBuilder> IntoContent for ExpandElements<T, F> {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        let mut result = vec![];
        for builder in self {
            let entity = world.spawn_empty().id();
            result.push(entity.clone());
            builder.with_entity(entity)(world);
        }
        result
    }
}

impl IntoContent for Vec<ElementsBuilder> {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        let mut result = vec![];
        for builder in self {
            let entity = world.spawn_empty().id();
            result.push(entity.clone());
            builder.with_entity(entity)(world);
        }
        result
    }
}

impl IntoContent for ElementsBuilder {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity> {
        self.with_entity(parent)(world);
        vec![parent]
    }
}

pub struct ExpandElements<I: Iterator, F: Fn(I::Item) -> ElementsBuilder> {
    mapper: F,
    previous: I,
}

impl<I, F> Iterator for ExpandElements<I, F>
where
    I: Iterator,
    F: Fn(I::Item) -> ElementsBuilder,
{
    type Item = ElementsBuilder;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(x) = self.previous.next() {
            return Some((self.mapper)(x));
        }
        None
    }
}

pub trait ExpandElementsExt: Iterator {
    fn elements<F: Fn(Self::Item) -> ElementsBuilder>(self, mapper: F) -> ExpandElements<Self, F>
    where
        Self: Sized,
    {
        ExpandElements {
            mapper,
            previous: self,
        }
    }
}

impl<I: Iterator> ExpandElementsExt for I {}
