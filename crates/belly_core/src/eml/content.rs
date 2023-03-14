use crate::{
    element::{Element, TextElementBundle},
    eml::Eml,
    relations::{
        bind::{BindableSource, BindableTarget, FromComponent, FromResourceWithTransformer},
        RelationsSystems,
    },
    to,
};
use bevy::{
    ecs::query::{QueryItem, WorldQuery},
    prelude::*,
};
use std::any::TypeId;

pub trait IntoContent: Sized {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity>;
}

pub trait UpdateContent: Sized {
    type Query: WorldQuery;
    fn update_content(item: QueryItem<Self::Query>, value: &Self);
}

impl IntoContent for String {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        let text = Text::from_section(self, Default::default());
        let entity = world
            .spawn(TextElementBundle {
                text: TextBundle { text, ..default() },
                ..default()
            })
            .id();
        vec![entity]
    }
}

impl UpdateContent for String {
    type Query = &'static mut Text;
    fn update_content(mut item: QueryItem<Self::Query>, value: &Self) {
        item.sections[0].value = value.clone();
    }
}

impl IntoContent for &str {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity> {
        self.to_string().into_content(parent, world)
    }
}
impl UpdateContent for &str {
    type Query = &'static mut Text;
    fn update_content(mut item: QueryItem<Self::Query>, value: &Self) {
        item.sections[0].value = value.to_string();
    }
}

#[derive(Component)]
pub struct BindContent<S: BindableSource + IntoContent + Clone> {
    value: S,
}
impl<R: Component, S: BindableTarget + Default + IntoContent + Clone + UpdateContent> IntoContent
    for FromComponent<R, S>
{
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        let Some(entity) = S::default().into_content(_parent, world).first().copied() else {
            return vec![];
        };
        let bind = self >> to!(entity, BindContent<S>:value);
        bind.write(world);
        world.entity_mut(entity).insert(BindContent {
            value: S::default(),
        });
        let systems = world.get_resource_or_insert_with(RelationsSystems::default);
        systems
            .0
            .add_custom_system(TypeId::of::<BindContent<S>>(), update_content_system::<S>);
        vec![entity]
    }
}

impl<
        R: Resource,
        S: BindableSource,
        T: BindableTarget + Clone + Default + IntoContent + UpdateContent,
    > IntoContent for FromResourceWithTransformer<R, S, T>
{
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity> {
        let Some(entity) = T::default().into_content(parent, world).first().copied() else {
            return vec![];
        };
        let bind = self >> to!(entity, BindContent<T>:value);
        bind.write(world);
        world.entity_mut(entity).insert(BindContent {
            value: T::default(),
        });
        let systems = world.get_resource_or_insert_with(RelationsSystems::default);
        systems
            .0
            .add_custom_system(TypeId::of::<BindContent<T>>(), update_content_system::<T>);
        vec![entity]
    }
}

fn update_content_system<T: UpdateContent + IntoContent + BindableSource>(
    mut binds: Query<(T::Query, &BindContent<T>), Changed<BindContent<T>>>,
) {
    for (item, bind) in binds.iter_mut() {
        T::update_content(item, &bind.value)
    }
}

impl IntoContent for Vec<Entity> {
    fn into_content(self, _parent: Entity, _world: &mut World) -> Vec<Entity> {
        self
    }
}

impl<T: Iterator, F: Fn(T::Item) -> Eml> IntoContent for ExpandElements<T, F> {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        self.into_iter()
            .map(|builder| builder.build(world))
            .collect()
    }
}

impl IntoContent for Vec<Eml> {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        self.into_iter()
            .map(|builder| builder.build(world))
            .collect()
    }
}

impl IntoContent for Eml {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        vec![self.build(world)]
    }
}

pub struct ExpandElements<I: Iterator, F: Fn(I::Item) -> Eml> {
    mapper: F,
    previous: I,
}

impl<I, F> Iterator for ExpandElements<I, F>
where
    I: Iterator,
    F: Fn(I::Item) -> Eml,
{
    type Item = Eml;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(x) = self.previous.next() {
            return Some((self.mapper)(x));
        }
        None
    }
}

pub trait ExpandElementsExt: Iterator {
    fn elements<F: Fn(Self::Item) -> Eml>(self, mapper: F) -> ExpandElements<Self, F>
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
