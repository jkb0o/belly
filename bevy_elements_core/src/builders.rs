use bevy::ui::FocusPolicy;
use bevy::utils::HashMap;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;

use crate::context::*;
use crate::element::*;
use crate::tags;
use crate::AttributeValue;
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::*;
use tagstr::*;

pub(crate) fn build_element(mut ctx: ResMut<BuildingContext>, mut commands: Commands) {
    commands
        .entity(ctx.element)
        .insert(NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            ..Default::default()
        })
        .push_children(&ctx.content());
}

pub(crate) fn default_postprocessor(
    mut ctx: ResMut<BuildingContext>,
    mut commands: Commands,
    mut elements: Query<&mut Element>,
) {
    let tag = ctx.name;
    let element = ctx.element;
    let mut commands = commands.entity(element.clone());
    let mut params = ctx.params();
    commands.insert(Name::new(tag.to_string()));

    if let Some(attr_commands) = params.commands(tags::with()) {
        attr_commands(&mut commands);
    }
    if let Ok(mut element) = elements.get_mut(element) {
        element.name = Some(tag);
        element.id = params.id();
        element.classes.extend(params.classes());
        element.styles.extend(params.styles());
    } else {
        let element = Element {
            name: Some(tag),
            id: params.id(),
            classes: params.classes(),
            styles: params.styles(),
            ..default()
        };
        commands.insert(element);
    }
    let focus_policy = match params.drop_variant(tag!("interactable")) {
        Some(AttributeValue::Empty) => Some(FocusPolicy::Pass),
        Some(AttributeValue::String(s)) if &s == "block" => Some(FocusPolicy::Block),
        Some(AttributeValue::String(s)) if &s == "pass" => Some(FocusPolicy::Pass),
        _ => None,
    };
    if let Some(policy) = focus_policy {
        commands.insert(policy);
        commands.insert(Interaction::default());
    }
}

#[derive(Clone)]
pub struct ElementBuilder {
    system: Rc<RefCell<BoxedSystem<(), ()>>>,
    postprocess: bool,
}

unsafe impl Send for ElementBuilder {}
unsafe impl Sync for ElementBuilder {}

impl ElementBuilder {
    pub fn from_boxed(mut boxed: BoxedSystem<(), ()>, world: &mut World) -> ElementBuilder {
        boxed.initialize(world);
        ElementBuilder {
            postprocess: false,
            system: Rc::new(RefCell::new(boxed)),
        }
    }
    pub(crate) fn new<Params, S: IntoSystem<(), (), Params>>(
        world: &mut World,
        builder: S,
    ) -> ElementBuilder {
        let mut system = IntoSystem::into_system(builder);
        system.initialize(world);
        ElementBuilder {
            postprocess: false,
            system: Rc::new(RefCell::new(Box::new(system))),
        }
    }
    pub fn with_postprocessing(mut self) -> Self {
        self.postprocess = true;
        self
    }
    pub fn build(&self, world: &mut World) {
        let mut system = self.system.borrow_mut();
        system.run((), world);
        system.apply_buffers(world);
        if !self.postprocess {
            return;
        }
        let processors = world
            .get_resource_mut::<ElementPostProcessors>()
            .unwrap()
            .0
            .clone();
        for postprocessor in processors.borrow().iter() {
            postprocessor.build(world)
        }
    }
}

#[derive(Default, Resource)]
pub struct ElementPostProcessors(pub(crate) Rc<RefCell<Vec<ElementBuilder>>>);
unsafe impl Send for ElementPostProcessors {}
unsafe impl Sync for ElementPostProcessors {}

#[derive(Resource, Clone, Default)]
pub struct ElementBuilderRegistry(Arc<RwLock<HashMap<Tag, ElementBuilder>>>);

unsafe impl Send for ElementBuilderRegistry {}
unsafe impl Sync for ElementBuilderRegistry {}

impl ElementBuilderRegistry {
    pub fn new() -> ElementBuilderRegistry {
        ElementBuilderRegistry(Default::default())
    }

    pub fn get_builder(&self, name: Tag) -> Option<ElementBuilder> {
        self.0.read().unwrap().get(&name).map(|b| b.clone())
    }

    pub fn add_builder(&mut self, name: Tag, builder: ElementBuilder) {
        self.0.write().unwrap().insert(name, builder);
    }

    pub fn has_builder(&self, name: Tag) -> bool {
        self.0.read().unwrap().contains_key(&name)
    }
}

#[derive(Resource)]
struct WidgetBuilder<T> {
    builder: ElementBuilder,
    marker: PhantomData<T>,
}

pub trait Widget: Sized + Send + Sync + 'static {
    // fn build() -> Self;
    fn widget_builder(world: &mut World) -> ElementBuilder {
        if let Some(systemres) = world.get_resource::<WidgetBuilder<Self>>() {
            systemres.builder.clone()
        } else {
            let system = Self::get_system();
            let builder = ElementBuilder::from_boxed(system, world).with_postprocessing();
            world.insert_resource(WidgetBuilder {
                builder: builder.clone(),
                marker: PhantomData::<Self>,
            });
            builder
        }
    }
    fn get_system() -> BoxedSystem<(), ()>;
}

#[macro_export]
macro_rules! widget {
    ($typ:ident, $($($a:ident)+: $t:ty),* => $($body:tt)*) => {
        impl $crate::Widget for $typ {
            fn get_system() -> ::bevy::ecs::system::BoxedSystem<(), ()> {
                ::std::boxed::Box::new(
                    ::bevy::ecs::system::IntoSystem::into_system(
                        move |$($($a)+: $t),*| $($body)*
                    )
                )
            }
        }
    };
}
