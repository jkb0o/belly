pub mod colors;
pub mod enums;
pub mod impls;
pub mod parse;
mod style;
use std::any::{type_name, Any};
use std::sync::{Arc, RwLock};

pub use self::colors::*;
pub use self::style::StyleProperty;
pub use self::style::StylePropertyFunction;
pub use self::style::StylePropertyMethods;
pub use self::style::StylePropertyToken;
pub use self::style::ToRectMap;
use crate::tags::*;
use crate::{
    element::*,
    eml::Variant,
    ess::{ElementsBranch, StyleSheet, Styles},
    ElementsError,
};
use bevy::ui::UiSystem;
use bevy::{
    ecs::query::{QueryItem, ReadOnlyWorldQuery, WorldQuery},
    prelude::*,
    utils::HashMap,
};
use itertools::Itertools;

pub struct PropertyPlugin;
impl Plugin for PropertyPlugin {
    fn build(&self, app: &mut App) {
        // general
        app.register_property::<impls::BackgroundColorProperty>();
        app.register_property::<impls::ZIndexProperty>();

        // layout control
        app.register_compound_property::<impls::layout_control::PositionProperty>();
        app.register_property::<impls::layout_control::PositionTypeProperty>();
        app.register_property::<impls::layout_control::LeftProperty>();
        app.register_property::<impls::layout_control::RightProperty>();
        app.register_property::<impls::layout_control::TopProperty>();
        app.register_property::<impls::layout_control::BottomProperty>();
        app.register_property::<impls::layout_control::OverflowProperty>();
        app.register_property::<impls::layout_control::DisplayProperty>();

        // flex container
        app.register_property::<impls::flex_container::FlexDirectionProperty>();
        app.register_property::<impls::flex_container::FlexWrapProperty>();
        app.register_property::<impls::flex_container::AlignItemsProperty>();
        app.register_property::<impls::flex_container::AlignContentProperty>();
        app.register_property::<impls::flex_container::JustifyContentProperty>();

        // flex item
        app.register_property::<impls::flex_item::AlignSelfProperty>();
        app.register_property::<impls::flex_item::FlexGrowProperty>();
        app.register_property::<impls::flex_item::FlexShrinkProperty>();
        app.register_property::<impls::flex_item::FlexBasisProperty>();

        // spacing
        app.register_compound_property::<impls::spacing::PaddingProperty>();
        app.register_property::<impls::spacing::PaddingLeftProperty>();
        app.register_property::<impls::spacing::PaddingRightProperty>();
        app.register_property::<impls::spacing::PaddingTopProperty>();
        app.register_property::<impls::spacing::PaddingBottomProperty>();
        app.register_compound_property::<impls::spacing::MarginProperty>();
        app.register_property::<impls::spacing::MarginLeftProperty>();
        app.register_property::<impls::spacing::MarginRightProperty>();
        app.register_property::<impls::spacing::MarginTopProperty>();
        app.register_property::<impls::spacing::MarginBottomProperty>();
        app.register_compound_property::<impls::spacing::BorderProperty>();
        app.register_property::<impls::spacing::BorderLeftProperty>();
        app.register_property::<impls::spacing::BorderRightProperty>();
        app.register_property::<impls::spacing::BorderTopProperty>();
        app.register_property::<impls::spacing::BorderBottomProperty>();
        app.register_property::<impls::spacing::ColumnGapProperty>();
        app.register_property::<impls::spacing::RowGapProperty>();

        // size constraints
        app.register_property::<impls::size_constraints::WidthProperty>();
        app.register_property::<impls::size_constraints::HeightProperty>();
        app.register_property::<impls::size_constraints::MinWidthProperty>();
        app.register_property::<impls::size_constraints::MinHeightProperty>();
        app.register_property::<impls::size_constraints::MaxWidthProperty>();
        app.register_property::<impls::size_constraints::MaxHeightProperty>();
        app.register_property::<impls::size_constraints::AspectRatioProperty>();

        // text
        app.register_property::<impls::text::ColorProperty>();
        app.register_property::<impls::text::FontProperty>();
        app.register_property::<impls::text::FontSizeProperty>();

        // stylebox
        app.register_compound_property::<impls::stylebox::StyleboxProperty>();
        app.register_property::<impls::stylebox::StyleboxSourceProperty>();
        app.register_property::<impls::stylebox::StyleboxModulateProperty>();
        app.register_property::<impls::stylebox::StyleboxRegionProperty>();
        app.register_property::<impls::stylebox::StyleboxSliceProperty>();
        app.register_property::<impls::stylebox::StyleboxWidthProperty>();

        // grid
        app.register_property::<impls::grid::GridAutoColumnsProperty>();
        app.register_property::<impls::grid::GridAutoRowsProperty>();
        app.register_property::<impls::grid::GridTemplateColumnsProperty>();
        app.register_property::<impls::grid::GridTemplateRowsProperty>();
        app.register_property::<impls::grid::GridRowProperty>();
        app.register_property::<impls::grid::GridColumnProperty>();
        app.register_property::<impls::grid::GridAutoFlowProperty>();
        app.register_property::<impls::grid::JustifyItemsProperty>();
        app.register_property::<impls::grid::JustifySelfProperty>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct ApplyStyleProperties;

pub struct ManagedPropertyValue(StyleProperty);

pub fn managed() -> PropertyValue {
    PropertyValue::new_managed()
}

pub fn managed_default(default: &str) -> PropertyValue {
    match StyleProperty::try_from(default) {
        Ok(val) => PropertyValue::new_managed_with_default(val),
        Err(e) => {
            error!("Error parsing default managed value '{default}': {e}");
            PropertyValue::new_managed()
        }
    }
}
#[derive(Deref, Debug)]
pub struct PropertyValue(Box<dyn Any + Send + Sync + 'static>);

impl PropertyValue {
    pub fn new<T: Any + Send + Sync + 'static>(value: T) -> PropertyValue {
        PropertyValue(Box::new(value))
    }

    pub fn new_managed() -> PropertyValue {
        PropertyValue::new(ManagedPropertyValue(Default::default()))
    }
    pub fn new_managed_with_default(default: StyleProperty) -> PropertyValue {
        PropertyValue::new(ManagedPropertyValue(default))
    }

    pub fn is_managed(&self) -> bool {
        self.0.is::<ManagedPropertyValue>()
    }

    pub fn managed_default(&self) -> Option<&StyleProperty> {
        self.0.downcast_ref::<ManagedPropertyValue>().and_then(|s| {
            if s.0.is_empty() {
                None
            } else {
                Some(&s.0)
            }
        })
    }
}

impl From<PropertyValue> for Variant {
    fn from(v: PropertyValue) -> Self {
        Variant::Property(v)
    }
}

/// Determines how a property should be parsed into exact value
pub trait PropertyParser<T: Default + Any + Send + Sync> {
    fn parse(value: &StyleProperty) -> Result<T, ElementsError>;
}

/// Determines how a property should interact and modify the [ecs world](`bevy::prelude::World`).
///
/// Each implementation of this trait should be registered with [`RegisterProperty`](crate::RegisterProperty) trait, where
/// will be converted into a `system` and run whenever a matched, specified by [`name()`](`Property::name()`) property is found.
///
/// These are the associated types that must by specified by implementors:
/// - [`Cache`](Property::Cache) is a cached value to be applied by this trait.
/// On the first time the `system` runs it'll call [`parse`](`Property::parse`) and cache the value.
/// Subsequential runs will only fetch the cached value.
/// - [`Components`](Property::Components) is which components will be send to [`apply`](`Property::apply`) function whenever a
/// valid cache exists and a matching property was found on any sheet rule. Check [`WorldQuery`] for more.
/// - [`Filters`](Property::Filters) is used to filter which entities will be applied the property modification.
/// Entities are first filtered by [`selectors`](`Selector`), but it can be useful to also ensure some behavior for safety reasons,
/// like only inserting [`TextAlignment`](bevy::prelude::TextAlignment) if the entity also has a [`Text`](bevy::prelude::Text) component.
///  Check [`WorldQuery`] for more.
///
/// These are tree functions required to be implemented:
/// - [`name`](Property::name) indicates which property name should matched for.
/// - [`parse`](Property::parse) parses the [`PropertyValues`] into the [`Cache`](Property::Cache) value to be reused across multiple entities.
/// - [`apply`](Property::apply) applies on the given [`Components`](Property::Components) the [`Cache`](Property::Cache) value.
/// Additionally, an [`AssetServer`] and [`Commands`] parameters are provided for more complex use cases.
///
/// Also, there one function which have default implementations:
/// - [`apply_system`](Property::apply_system) is a [`system`](https://docs.rs/bevy_ecs/0.8.1/bevy_ecs/system/index.html) which interacts with
/// [ecs world](`bevy::prelude::World`) and call the [`apply`](Property::apply) function on every matched entity.
pub trait Property: Default + Sized + Send + Sync + 'static {
    /// The item value type to be applied by property.
    type Item: Default + Any + Send + Sync;
    /// Which components should be queried when applying the modification. Check [`WorldQuery`] for more.
    type Components: WorldQuery;
    /// Filters conditions to be applied when querying entities by this property. Check [`WorldQuery`] for more.
    type Filters: ReadOnlyWorldQuery;
    /// Associate [`PropertyParser`] with [`Property`]
    type Parser: PropertyParser<Self::Item>;

    /// Indicates which property name should matched for. Must match the same property name as on `css` file.
    ///
    /// For compliance, use always `lower-case` and `kebab-case` names.
    fn name() -> Tag;

    fn affects_virtual_elements() -> bool {
        false
    }

    fn docstring() -> &'static str {
        ""
    }

    /// Parses the [`PropertyValues`] into the [`Cache`](Property::Cache) value to be reused across multiple entities.
    ///
    /// This function is called only once, on the first time a matching property is found while applying style rule.
    /// If an error is returned, it is also cached so no more attempt are made.
    // fn parse(values: &StyleProperty) -> Result<Self::Item, ElementsError>;

    fn transform(variant: Variant) -> Result<PropertyValue, ElementsError> {
        match variant {
            Variant::Style(p) => Self::Parser::parse(&p).map(|p| PropertyValue::new(p)),
            Variant::String(s) => StyleProperty::try_from(s)
                .and_then(|p| Self::Parser::parse(&p))
                .map(|v| PropertyValue::new(v)),
            Variant::Boxed(b) => Ok(PropertyValue::new(*b.downcast::<Self::Item>().map_err(
                |e| {
                    ElementsError::InvalidPropertyValue(format!(
                        "Can't downcast {:?} to {}",
                        e,
                        type_name::<Self::Item>()
                    ))
                },
            )?)),
            Variant::Property(p) => Ok(p),
            variant => Err(ElementsError::InvalidPropertyValue(format!(
                "Don't know how to transform {:?} into {} PropertyValue",
                variant,
                type_name::<Self::Item>()
            ))),
        }
    }

    /// Applies on the given [`Components`](Property::Components) the [`Cache`](Property::Cache) value.
    /// Additionally, an [`AssetServer`] and [`Commands`] parameters are provided for more complex use cases.
    ///
    /// If mutability is desired while applying the changes, declare [`Components`](Property::Components) as mutable.
    fn apply(
        cache: &Self::Item,
        components: QueryItem<Self::Components>,
        asset_server: &AssetServer,
        commands: &mut Commands,
        entity: Entity,
    );

    /// The [`system`](https://docs.rs/bevy_ecs/0.8.1/bevy_ecs/system/index.html) which interacts with
    /// [ecs world](`bevy::prelude::World`) and call [`apply`](Property::apply) function on every matched entity.
    ///
    /// The default implementation will cover most use cases, by just implementing [`apply`](Property::apply)
    fn apply_defaults(
        // mut cached_properties: Local<CachedProperties<Self>>,
        mut components: Query<(Entity, Self::Components), (Changed<Element>, Self::Filters)>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        styles: Res<Styles>,
        stylesheets: Res<Assets<StyleSheet>>,
        parents: Query<&Parent>,
        elements: Query<&Element>,
    ) {
        if components.is_empty() {
            return;
        }
        // info!("[prop] changed {}", components.iter().count());
        // TODO: this should be cached
        let mut rules: Vec<_> = styles
            .iter()
            .filter_map(|h| stylesheets.get(h))
            .flat_map(|s| s.iter())
            .filter(|r| r.properties.contains_key(&Self::name()))
            .collect();
        rules.sort_by_key(|r| -r.selector.weight);

        for (entity, components) in components.iter_mut() {
            let Ok(element) = elements.get(entity) else {
                continue;
            };
            if element.is_virtual() && !Self::affects_virtual_elements() {
                continue;
            }

            // extract default value
            let mut element_with_default = element;
            let mut entity_with_default = entity;
            let mut default = None;
            loop {
                if !element_with_default.is_virtual() {
                    default = element_with_default.styles.get(&Self::name());
                    break;
                }
                if let Ok(parent) = parents.get(entity_with_default) {
                    entity_with_default = parent.get();
                    if let Ok(element) = elements.get(entity_with_default) {
                        element_with_default = element;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            if default.is_some() && default.unwrap().is_managed() {
                continue;
            }

            // compute branch
            let mut branch = ElementsBranch::new();
            let mut tail = entity;
            while let Ok(element) = elements.get(tail) {
                if !element.is_virtual() {
                    branch.insert(element);
                }
                if let Ok(parent) = parents.get(tail) {
                    tail = parent.get();
                } else {
                    break;
                }
            }
            let property = default.or_else(|| {
                rules
                    .iter()
                    .filter_map(|r| {
                        if let Some(depth) = r.selector.match_depth(&branch) {
                            Some((
                                r.properties.get(&Self::name()).unwrap(),
                                depth,
                                r.selector.weight,
                            ))
                        } else {
                            None
                        }
                    })
                    .group_by(|(_prop, _depth, weight)| *weight)
                    .into_iter()
                    .map(|(_, group)| group)
                    .next()
                    .map(|properties| {
                        let mut variants = properties.collect::<Vec<_>>();
                        variants.sort_by_key(|(_prop, depth, _weight)| -(*depth as i16));
                        let (value, _depth, _weight) = variants.pop().unwrap();
                        value
                    })
            });

            if let Some(property) = property {
                if let Some(property) = property.downcast_ref::<Self::Item>() {
                    Self::apply(property, components, &asset_server, &mut commands, entity);
                } else {
                    error!(
                        "Unable to apply {} property: inconsistent Variant {:?}",
                        Self::name(),
                        property
                    );
                }
            }
        }
    }
}

pub trait CompoundProperty: Default + Sized + Send + Sync + 'static {
    fn name() -> Tag;
    fn docstring() -> &'static str {
        ""
    }
    fn extract(value: Variant) -> Result<HashMap<Tag, PropertyValue>, ElementsError>;
    fn error(message: String) -> Result<HashMap<Tag, PropertyValue>, ElementsError> {
        Err(ElementsError::InvalidPropertyValue(message))
    }
}

pub(crate) type TransformProperty = fn(Variant) -> Result<PropertyValue, ElementsError>;
#[derive(Default, Clone, Resource)]
pub struct PropertyTransformer(Arc<RwLock<HashMap<Tag, TransformProperty>>>);
unsafe impl Send for PropertyTransformer {}
unsafe impl Sync for PropertyTransformer {}
impl PropertyTransformer {
    #[cfg(test)]
    pub(crate) fn new(rules: HashMap<Tag, TransformProperty>) -> PropertyTransformer {
        PropertyTransformer(Arc::new(RwLock::new(rules)))
    }
    pub(crate) fn transform(
        &self,
        name: Tag,
        value: Variant,
    ) -> Result<PropertyValue, ElementsError> {
        self.0
            .read()
            .unwrap()
            .get(&name)
            .ok_or(ElementsError::UnsupportedProperty(name.to_string()))
            .and_then(|transform| transform(value))
    }
}

pub(crate) type ExtractProperty = fn(Variant) -> Result<HashMap<Tag, PropertyValue>, ElementsError>;
#[derive(Default, Clone, Resource)]
pub struct PropertyExtractor(Arc<RwLock<HashMap<Tag, ExtractProperty>>>);
unsafe impl Send for PropertyExtractor {}
unsafe impl Sync for PropertyExtractor {}
impl PropertyExtractor {
    #[cfg(test)]
    pub(crate) fn new(rules: HashMap<Tag, ExtractProperty>) -> PropertyExtractor {
        PropertyExtractor(Arc::new(RwLock::new(rules)))
    }
    pub(crate) fn is_compound_property(&self, name: Tag) -> bool {
        self.0.read().unwrap().contains_key(&name)
    }

    pub(crate) fn extract(
        &self,
        name: Tag,
        value: Variant,
    ) -> Result<HashMap<Tag, PropertyValue>, ElementsError> {
        self.0
            .read()
            .unwrap()
            .get(&name)
            .ok_or(ElementsError::UnsupportedProperty(name.to_string()))
            .and_then(|extractor| extractor(value))
    }
}

pub trait RegisterProperty {
    fn register_property<T: Property + 'static>(&mut self) -> &mut Self;
    fn register_compound_property<T: CompoundProperty + 'static>(&mut self) -> &mut Self;
}

impl RegisterProperty for bevy::prelude::App {
    fn register_property<T: Property + 'static>(&mut self) -> &mut Self {
        self.world
            .get_resource_or_insert_with(PropertyTransformer::default)
            .0
            .write()
            .unwrap()
            .entry(T::name())
            .and_modify(|_| panic!("Property `{}` already registered.", T::name()))
            .or_insert(T::transform);
        self.add_systems(
            PostUpdate,
            T::apply_defaults
                .in_set(ApplyStyleProperties)
                .after(InvalidateElements)
                .before(UiSystem::Layout),
        );
        self
    }

    fn register_compound_property<T: CompoundProperty + 'static>(&mut self) -> &mut Self {
        self.world
            .get_resource_or_insert_with(PropertyExtractor::default)
            .0
            .write()
            .unwrap()
            .entry(T::name())
            .and_modify(|_| panic!("CompoundProperty `{}` already registered", T::name()))
            .insert(T::extract);
        self
    }
}

#[macro_export]
macro_rules! style_property {
    ( $(#[doc = $s:literal])*
      $typename:ident($prop_name:literal) {
        Default = $default:literal;
        Item = $item:ty;
        Components = $components:ty;
        Filters = $filters:ty;
        AffectsVirtual = $affects_virtual:literal;
        Parser = $parser:ty;
        Apply = | $value:ident, $component:ident, $assets:ident, $commands:ident, $entity:ident |
            $body:expr;
    }) => {
        #[derive(Default)]
        $(#[doc = $s])*
        #[doc = concat!(" <!-- @property-name=", $prop_name, " -->")]
        #[doc = concat!(" <!-- @property-default=", $default, " -->")]
        pub struct $typename;
        impl $crate::ess::Property for $typename {
            type Item = $item;
            type Components = $components;
            type Filters = $filters;
            type Parser = $parser;

            fn name() -> $crate::Tag {
                $crate::tag!($prop_name)
            }

            fn affects_virtual_elements() -> bool {
                $affects_virtual
            }

            fn apply(
                $value: &Self::Item,
                #[allow(unused_mut)]
                mut $component: ::bevy::ecs::query::QueryItem<Self::Components>,
                $assets: &::bevy::prelude::AssetServer,
                $commands: &mut ::bevy::prelude::Commands,
                $entity: ::bevy::prelude::Entity,
            ) {
                $body
            }
        }
    };
    ( $(#[doc = $s:literal])*
      $typename:ident($prop_name:literal) {
        Default = $default:literal;
        Item = $item:ty;
        Components = $components:ty;
        Filters = $filters:ty;
        Parser = $parser:ty;
        Apply = | $value:ident, $component:ident, $assets:ident, $commands:ident, $entity:ident |
            $body:expr;
    }) => { $crate::style_property! {
        $(#[doc = $s])*
        $typename($prop_name) {
            Default = $default;
            Item = $item;
            Components = $components;
            Filters = $filters;
            AffectsVirtual = false;
            Parser = $parser;
            Apply = | $value, $component, $assets, $commands, $entity |
                $body;
        }
    }};
}

#[macro_export]
macro_rules! compound_style_property {
    (   $(#[doc = $s:literal])*
        $typename:ident($prop_name:literal, $value:ident)
            $body:expr
    ) => {
        #[derive(Default)]
        $(#[doc = $s])*
        #[doc = concat!(" <!-- @property-name=", $prop_name, " -->")]
        pub struct $typename;
        impl $crate::ess::CompoundProperty for $typename {
            fn name() -> $crate::Tag {
                $crate::tag!($prop_name)
            }
            fn extract($value: $crate::eml::Variant) -> Result<::bevy::utils::HashMap<$crate::Tag, $crate::ess::PropertyValue>, $crate::ElementsError> {
                $body
            }
        }
    }
}

#[cfg(test)]
mod test {
    use smallvec::SmallVec;

    use super::*;

    #[test]
    fn parse_value() {
        let expected = StyleProperty(SmallVec::from_vec(vec![
            StylePropertyToken::Percentage(21f32.into()),
            StylePropertyToken::Dimension(22f32.into(), "px".into()),
        ]));
        let value = "21% 22px";
        assert_eq!(Ok(expected), value.try_into());
    }
}
