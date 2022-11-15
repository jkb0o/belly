use std::any::TypeId;
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;

use bevy::ecs::component::ComponentId;
use bevy::utils::HashMap;
use bevy::{
    ecs::system::EntityCommands,
    prelude::*
};
use bevy_inspector_egui::egui::mutex::RwLock;
use property::PropertyValues;

pub mod attributes;
pub mod tags;
pub mod context;
pub mod builders;
pub mod element;
pub mod property;
pub mod css;

pub struct BsxPlugin;

use crate::builders::*;
pub use context::BuildingContext;
pub use property::Property;
pub use tagstr::*;
pub use context::IntoContent;
pub use context::ExpandElements;
pub use context::ExpandElementsExt;

impl Plugin for BsxPlugin {
    fn build(&self, app: &mut App) {
        app.register_text_element_builder(build_text)
            .register_element_builder("el", build_element)
            .register_elements_postprocessor(default_postprocessor)
            .insert_resource(DefaultFont::default());

        // TODO: may be desabled with feature
        app.add_startup_system(setup_default_font);

        register_properties(app);
    }
}

fn register_properties(app: &mut bevy::prelude::App) {
    use property::impls::*;

    app.register_property::<DisplayProperty>();
    app.register_property::<PositionTypeProperty>();
    app.register_property::<DirectionProperty>();
    app.register_property::<FlexDirectionProperty>();
    app.register_property::<FlexWrapProperty>();
    app.register_property::<AlignItemsProperty>();
    app.register_property::<AlignSelfProperty>();
    app.register_property::<AlignContentProperty>();
    app.register_property::<JustifyContentProperty>();
    app.register_property::<OverflowProperty>();

    app.register_property::<LeftProperty>();
    app.register_property::<RightProperty>();
    app.register_property::<TopProperty>();
    app.register_property::<BottomProperty>();
    app.register_property::<MarginLeftProperty>();
    app.register_property::<PaddingLeftProperty>();
    app.register_property::<WidthProperty>();
    app.register_property::<HeightProperty>();
    app.register_property::<MinWidthProperty>();
    app.register_property::<MinHeightProperty>();
    app.register_property::<MaxWidthProperty>();
    app.register_property::<MaxHeightProperty>();
    app.register_property::<FlexBasisProperty>();
    app.register_property::<FlexGrowProperty>();
    app.register_property::<FlexShrinkProperty>();
    app.register_property::<AspectRatioProperty>();

    app.register_property::<MarginProperty>();
    app.register_property::<PaddingProperty>();
    app.register_property::<BorderProperty>();

    app.register_property::<FontColorProperty>();
    app.register_property::<FontProperty>();
    app.register_property::<FontSizeProperty>();
    app.register_property::<VerticalAlignProperty>();
    app.register_property::<HorizontalAlignProperty>();
    app.register_property::<TextContentProperty>();

    app.register_property::<UiColorProperty>();
}

#[derive(Debug)]
pub enum ElementsError {
    /// An unsupported selector was found on a style sheet rule.
    UnsupportedSelector,
    /// An unsupported property was found on a style sheet rule.
    UnsupportedProperty(String),
    /// An invalid property value was found on a style sheet rule.
    InvalidPropertyValue(String),
    /// An invalid selector was found on a style sheet rule.
    InvalidSelector,
    /// An unexpected token was found on a style sheet rule.
    UnexpectedToken(String),
}

impl Error for ElementsError {}

impl Display for ElementsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementsError::UnsupportedSelector => {
                write!(f, "Unsupported selector")
            }
            ElementsError::UnsupportedProperty(p) => write!(f, "Unsupported property: {}", p),
            ElementsError::InvalidPropertyValue(p) => write!(f, "Invalid property value: {}", p),
            ElementsError::InvalidSelector => write!(f, "Invalid selector"),
            ElementsError::UnexpectedToken(t) => write!(f, "Unexpected token: {}", t),
        }
    }
}

pub trait WithElements {
    fn with_elements(&mut self, elements: ElementsBuilder) -> &mut Self;
}

impl<'w, 's, 'a> WithElements for EntityCommands<'w, 's, 'a> {
    fn with_elements(&mut self, elements: ElementsBuilder) -> &mut Self {
        let entity = self.id();
        self.commands().add(elements.with_entity(entity));
        self
    }
}

pub trait RegisterElementBuilder {
    fn register_element_builder<Params, D: IntoSystem<(), (), Params>>(
        &mut self,
        name: &'static str,
        builder: D,
    ) -> &mut Self;

    
    fn register_elements_postprocessor<Params, D: IntoSystem<(), (), Params>>(
        &mut self,
        builder: D,
    ) -> &mut Self;
}

pub (crate) trait RegisterElementBuilderInternal {
    fn register_text_element_builder<Params, D: IntoSystem<(), (), Params>> (
        &mut self,
        builder: D,
    ) -> &mut Self;
}



impl RegisterElementBuilder for App {
    fn register_element_builder<Params, D: IntoSystem<(), (), Params>>(
        &mut self,
        name: &'static str,
        builder: D,
    ) -> &mut Self {
        let builder = ElementBuilder::new(&mut self.world, builder).with_postprocessing();
        self.world
            .get_resource_or_insert_with::<ElementBuilderRegistry>(ElementBuilderRegistry::new)
            .add_builder(name.into(), builder);
        self
    }

    fn register_elements_postprocessor<Params, D: IntoSystem<(), (), Params>>(
        &mut self,
        builder: D,
    ) -> &mut Self {
        let builder = ElementBuilder::new(&mut self.world, builder);
        self.world
            .get_resource_or_insert_with::<ElementPostProcessors>(ElementPostProcessors::default).0.borrow_mut()
            .push(builder);
        self

    }
}

impl RegisterElementBuilderInternal for App {
    fn register_text_element_builder<Params, D: IntoSystem<(), (), Params>> (
        &mut self,
        builder: D,
    ) -> &mut Self {
        let builder = ElementBuilder::new(&mut self.world, builder);
        self.world.insert_resource(TextElementBuilder(builder));
        self
    }
}

pub struct ElementsBuilder {
    pub builder: Box<dyn FnOnce(&mut World, Entity) + Sync + Send>,
}

impl ElementsBuilder {
    pub fn new<T>(builder: T) -> Self
    where
        T: FnOnce(&mut World, Entity) + Sync + Send + 'static,
    {
        ElementsBuilder {
            builder: Box::new(builder),
        }
    }

    pub fn with_entity(self, entity: Entity) -> impl FnOnce(&mut World) {
        move |world: &mut World| {
            (self.builder)(world, entity);
        }
    }
}

pub (crate) type ValidateProperty =  Box<dyn Fn(&PropertyValues) -> Result<(), ElementsError>>;
#[derive(Default, Clone)]
pub (crate) struct PropertyValidator(Arc<RwLock<HashMap<Tag, ValidateProperty>>>);
impl PropertyValidator {
    pub (crate) fn new(rules: HashMap<Tag, ValidateProperty>) -> PropertyValidator {
        PropertyValidator(Arc::new(RwLock::new(rules)))
    }
    pub (crate) fn validate(&self, name: Tag, value: &PropertyValues) -> Result<(), ElementsError> {
        self.0.read().get(&name)
            .ok_or(ElementsError::UnsupportedProperty(name.to_string()))
            .and_then(|validator| validator(value))
    }
}

pub (crate) type ExtractProperty = Box<dyn Fn(PropertyValues) -> Result<HashMap<Tag, PropertyValues>, ElementsError>>;
#[derive(Default, Clone)]
pub (crate) struct PropertyExtractor(Arc<RwLock<HashMap<Tag, ExtractProperty>>>);
impl PropertyExtractor {
    pub (crate) fn new(rules: HashMap<Tag, ExtractProperty>) -> PropertyExtractor {
        PropertyExtractor(Arc::new(RwLock::new(rules)))
    }
    pub (crate) fn is_compound_property(&self, name: Tag) -> bool {
        self.0.read().contains_key(&name)
    }

    pub (crate) fn extract(&self, name: Tag, value: PropertyValues) -> Result<HashMap<Tag, PropertyValues>, ElementsError> {
        self.0.read().get(&name)
            .ok_or(ElementsError::UnsupportedProperty(name.to_string()))
            .and_then(|extractor| extractor(value))
    }
}


unsafe impl Send for PropertyValidator { }
unsafe impl Sync for PropertyValidator { }

/// Utility trait which adds the [`register_property`](RegisterProperty::register_property) function on [`App`](bevy::prelude::App) to add a [`Property`] parser.
///
/// You need to register only custom properties which implements [`Property`] trait.
pub trait RegisterProperty {
    fn register_property<T>(&mut self) -> &mut Self
    where
        T: Property + 'static;
}

impl RegisterProperty for bevy::prelude::App {
    fn register_property<T>(&mut self) -> &mut Self
    where
        T: Property + 'static,
    {
        self.world.get_resource_or_insert_with(PropertyValidator::default)
            .0.write().insert(T::name(), Box::new(|props| {
                T::validate(props)
            }));
        self.add_system(T::apply_defaults /* .label(EcssSystem::Apply) */);

        self
    }
}