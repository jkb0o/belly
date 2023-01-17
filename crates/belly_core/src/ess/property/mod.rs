pub mod colors;
pub mod impls;
mod style;
use std::any::{type_name, Any};

pub use self::colors::*;
pub use self::style::StyleProperty;
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
use bevy::{
    ecs::query::{QueryItem, ReadOnlyWorldQuery, WorldQuery},
    prelude::*,
    utils::HashMap,
};
use itertools::Itertools;

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

/// Maps which entities was selected by a [`Selector`]
// #[derive(Debug, Clone, Default, Deref, DerefMut)]
// pub struct SelectedEntities(HashMap<Selector, SmallVec<[Entity; 8]>>);

/// Maps sheets for each [`StyleSheetAsset`].
// #[derive(Debug, Clone, Default, Deref, DerefMut)]
// pub struct StyleSheetState(HashMap<Handle<StyleSheetAsset>, SelectedEntities>);

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
    fn parse(values: &StyleProperty) -> Result<Self::Item, ElementsError>;

    fn transform(variant: Variant) -> Result<PropertyValue, ElementsError> {
        match variant {
            Variant::Style(p) => Self::parse(&p).map(|p| PropertyValue::new(p)),
            Variant::String(s) => StyleProperty::try_from(s)
                .and_then(|p| Self::parse(&p))
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
            let Ok(element) = elements.get(entity) else { continue };
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

#[cfg(test)]
mod test {
    use smallvec::SmallVec;

    use super::*;

    #[test]
    fn parse_value() {
        let expected = StyleProperty(SmallVec::from_vec(vec![
            StylePropertyToken::Percentage(21f32.into()),
            StylePropertyToken::Dimension(22f32.into()),
        ]));
        let value = "21% 22px";
        assert_eq!(Ok(expected), value.try_into());
    }
}
