use bevy::{ecs::query::QueryItem, prelude::*};

use crate::ElementsError;
use tagstr::*;

use super::{CompoundProperty, Property, StyleProperty, StylePropertyMethods};

pub(crate) use style::*;
pub(crate) use text::*;
pub(crate) use transform::*;

#[macro_export]
macro_rules! style_property {
    ( $(#[doc = $s:literal])*
      $typename:ident($prop_name:literal) {
        Item = $item:ty;
        Components = $components:ty;
        Filters = $filters:ty;
        Parse = |$tokens:ident| $parse:expr;
        Apply = | $value:ident, $component:ident, $assets:ident, $commands:ident, $entity:ident |
            $body:expr;
    }) => {
        #[derive(Default)]
        $(#[doc = $s])*
        struct $typename;
        impl $crate::ess::Property for $typename {
            type Item = $item;
            type Components = $components;
            type Filters = $filters;

            fn name() -> Tag {
                tag!($prop_name)
            }

            fn parse($tokens: &$crate::ess::StyleProperty) -> Result<Self::Item, $crate::ElementsError> {
                $parse
            }

            fn apply(
                $value: &Self::Item,
                mut $component: ::bevy::ecs::query::QueryItem<Self::Components>,
                $assets: &::bevy::prelude::AssetServer,
                $commands: &mut ::bevy::prelude::Commands,
                $entity: ::bevy::prelude::Entity,
            ) {
                $body
            }
            fn docstring() -> &'static str {
                concat!($($s,"\n",)*)
            }
        }
    }
}

#[macro_export]
macro_rules! compound_style_property {
    (   $(#[doc = $s:literal])*
        $typename:ident($prop_name:literal, $value:ident)
            $body:expr
    ) => {
        #[derive(Default)]
        $(#[doc = $s])*
        struct $typename;
        impl $crate::ess::CompoundProperty for $typename {
            fn name() -> Tag {
                tag!($prop_name)
            }
            fn extract($value: Variant) -> Result<::bevy::utils::HashMap<$crate::Tag, $crate::ess::PropertyValue>, $crate::ElementsError> {
                $body
            }
            fn docstring() -> &'static str {
                concat!($($s,"\n",)*)
            }
        }
    }
}

/// Impls for `bevy_ui` [`Style`] component
mod style {

    use bevy::utils::HashMap;

    use crate::{ess::PropertyValue, Variant};

    use super::*;
    // #[derive(Default)]
    // pub(crate) struct PaddingProperty;
    // impl CompoundProperty for PaddingProperty {
    //     fn name() -> Tag {
    //         tag!("padding")
    //     }
    //     fn extract(value: crate::Variant) -> Result<bevy::utils::HashMap<Tag, PropertyValue>, ElementsError> {
    //         let rect = match value {
    //             Variant::String(unparsed) => {
    //                 StyleProperty::try_from(unparsed).and_then(|prop| prop.rect())?
    //             },
    //             Variant::Style(prop) => prop.rect()?,
    //             variant => variant.take::<UiRect>().ok_or(ElementsError::InvalidPropertyValue(format!("Can't extract rect from variant")))?,

    //         };
    //         let mut props = HashMap::default();
    //         props.insert("padding-left".as_tag(), PropertyValue::new(rect.left));
    //         props.insert("padding-right".as_tag(), PropertyValue::new(rect.right));
    //         props.insert("padding-top".as_tag(), PropertyValue::new(rect.top));
    //         props.insert("padding-bottom".as_tag(), PropertyValue::new(rect.bottom));
    //         Ok(props)
    //     }
    // }

    /// Implements a new property property extractor for [`Style`] component which expects a rect value.
    macro_rules! impl_style_rect {
        ($name:expr, $struct:ident) => {
            #[doc = "Applies the `"]
            #[doc = $name]
            // #[doc = "` property on [Style::"]
            #[derive(Default)]
            pub(crate) struct $struct;

            impl CompoundProperty for $struct {
                fn name() -> Tag {
                    tag!($name)
                }

                fn extract(
                    value: crate::Variant,
                ) -> Result<bevy::utils::HashMap<Tag, PropertyValue>, ElementsError> {
                    let rect =
                        match value {
                            Variant::String(unparsed) => {
                                StyleProperty::try_from(unparsed).and_then(|prop| prop.rect())?
                            }
                            Variant::Style(prop) => prop.rect()?,
                            variant => variant.take::<UiRect>().ok_or(
                                ElementsError::InvalidPropertyValue(format!(
                                    "Can't extract rect from variant"
                                )),
                            )?,
                        };
                    let mut props = HashMap::default();
                    props.insert(
                        concat!($name, "-left").as_tag(),
                        PropertyValue::new(rect.left),
                    );
                    props.insert(
                        concat!($name, "-right").as_tag(),
                        PropertyValue::new(rect.right),
                    );
                    props.insert(
                        concat!($name, "-top").as_tag(),
                        PropertyValue::new(rect.top),
                    );
                    props.insert(
                        concat!($name, "-bottom").as_tag(),
                        PropertyValue::new(rect.bottom),
                    );
                    Ok(props)
                }
            }
        };
    }

    /// Implements a new property for [`Style`] component which expects a single value.
    macro_rules! impl_style_single_value {
        ($name:expr, $struct:ident, $cache:ty, $parse_func:ident, $style_prop:ident$(.$style_field:ident)*) => {
            #[doc = "Applies the `"]
            #[doc = $name]
            #[doc = "` property on [Style::"]
            #[doc = stringify!($style_prop)]
            $(#[doc = concat!("::",stringify!($style_field))])*
            #[doc = "](`Style`) field of all sections on matched [`Style`] components."]
            #[derive(Default)]
            pub(crate) struct $struct;

            impl Property for $struct {
                type Item = $cache;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> Tag {
                    tag!($name)
                }

                fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
                    values.$parse_func()
                }

                fn apply<'w>(
                    cache: &Self::Item,
                    mut components: QueryItem<Self::Components>,
                    _asset_server: &AssetServer,
                    _commands: &mut Commands,
                    _entity: Entity,
                ) {
                    components.$style_prop$(.$style_field)? = *cache;
                }
            }
        };
    }

    // Val properties
    impl_style_rect!("position", PositionProperty);
    impl_style_single_value!("left", LeftProperty, Val, val, position.left);
    impl_style_single_value!("right", RightProperty, Val, val, position.right);
    impl_style_single_value!("top", TopProperty, Val, val, position.top);
    impl_style_single_value!("bottom", BottomProperty, Val, val, position.bottom);
    impl_style_rect!("margin", MarginProperty);
    impl_style_single_value!("margin-left", MarginLeftProperty, Val, val, margin.left);
    impl_style_single_value!("margin-right", MarginRightProperty, Val, val, margin.right);
    impl_style_single_value!("margin-top", MarginTopProperty, Val, val, margin.top);
    impl_style_single_value!(
        "margin-bottom",
        MarginBottomProperty,
        Val,
        val,
        margin.bottom
    );
    impl_style_rect!("padding", PaddingProperty);
    impl_style_single_value!("padding-left", PaddingLeftProperty, Val, val, padding.left);
    impl_style_single_value!(
        "padding-right",
        PaddingRightProperty,
        Val,
        val,
        padding.right
    );
    impl_style_single_value!("padding-top", PaddingTopProperty, Val, val, padding.top);
    impl_style_single_value!(
        "padding-bottom",
        PaddingBottomProperty,
        Val,
        val,
        padding.bottom
    );
    impl_style_rect!("border", BorderProperty);
    impl_style_single_value!("border-left", BorderLeftProperty, Val, val, border.left);
    impl_style_single_value!("border-right", BorderRightProperty, Val, val, border.right);
    impl_style_single_value!("border-top", BorderTopProperty, Val, val, border.top);
    impl_style_single_value!(
        "border-bottom",
        BorderBottomProperty,
        Val,
        val,
        border.bottom
    );

    impl_style_single_value!("width", WidthProperty, Val, val, size.width);
    impl_style_single_value!("height", HeightProperty, Val, val, size.height);

    impl_style_single_value!("min-width", MinWidthProperty, Val, val, min_size.width);
    impl_style_single_value!("min-height", MinHeightProperty, Val, val, min_size.height);

    impl_style_single_value!("max-width", MaxWidthProperty, Val, val, max_size.width);
    impl_style_single_value!("max-height", MaxHeightProperty, Val, val, max_size.height);

    impl_style_single_value!("flex-basis", FlexBasisProperty, Val, val, flex_basis);

    impl_style_single_value!("flex-grow", FlexGrowProperty, f32, f32, flex_grow);
    impl_style_single_value!("flex-shrink", FlexShrinkProperty, f32, f32, flex_shrink);

    impl_style_single_value!(
        "aspect-ratio",
        AspectRatioProperty,
        Option<f32>,
        option_f32,
        aspect_ratio
    );

    /// Implements a new property for [`Style`] component which expects an enum.
    macro_rules! impl_style_enum {
        ($cache:ty, $name:expr, $struct:ident, $style_prop:ident, $($prop:expr => $variant:expr),+$(,)?) => {
            #[doc = "Applies the `"]
            #[doc = $name]
            #[doc = "` property on [Style::"]
            #[doc = stringify!($style_prop)]
            #[doc = "]("]
            #[doc = concat!("`", stringify!($cache), "`")]
            #[doc = ") field of all sections on matched [`Style`] components."]
            #[derive(Default)]
            pub(crate) struct $struct;

            impl Property for $struct {
                type Item = $cache;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> Tag {
                    tag!($name)
                }

                fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
                    if let Some(identifier) = values.identifier() {
                        use $cache::*;
                        // Chain if-let when `cargofmt` supports it
                        // https://github.com/rust-lang/rustfmt/pull/5203
                        match identifier {
                            $($prop => return Ok($variant)),+,
                            _ => (),
                        }
                    }

                    Err(ElementsError::InvalidPropertyValue(Self::name().to_string()))
                }

                fn apply<'w>(
                    cache: &Self::Item,
                    mut components: QueryItem<Self::Components>,
                    _asset_server: &AssetServer,
                    _commands: &mut Commands,
                    _entity: Entity,
                ) {
                    components.$style_prop = *cache;
                }
            }
        };
    }

    impl_style_enum!(Display, "display", DisplayProperty, display,
        "flex" => Flex,
        "none" => None
    );

    impl_style_enum!(PositionType, "position-type", PositionTypeProperty, position_type,
        "absolute" => Absolute,
        "relative" => Relative,
    );

    impl_style_enum!(Direction, "direction", DirectionProperty, direction,
        "inherit" => Inherit,
        "left-to-right" => LeftToRight,
        "right-to-left" => RightToLeft,
    );

    impl_style_enum!(FlexDirection, "flex-direction", FlexDirectionProperty, flex_direction,
        "row" => Row,
        "column" => Column,
        "row-reverse" => RowReverse,
        "column-reverse" => ColumnReverse,
    );

    impl_style_enum!(FlexWrap, "flex-wrap", FlexWrapProperty, flex_wrap,
        "no-wrap" => NoWrap,
        "wrap" => Wrap,
        "wrap-reverse" => WrapReverse,
    );

    impl_style_enum!(AlignItems, "align-items", AlignItemsProperty, align_items,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "baseline" => Baseline,
        "stretch" => Stretch,
    );

    impl_style_enum!(AlignSelf, "align-self", AlignSelfProperty, align_self,
        "auto" => Auto,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "baseline" => Baseline,
        "stretch" => Stretch,
    );

    impl_style_enum!(AlignContent, "align-content", AlignContentProperty, align_content,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "stretch" => Stretch,
        "space-between" => SpaceBetween,
        "space-around" => SpaceAround,
    );

    impl_style_enum!(JustifyContent, "justify-content", JustifyContentProperty, justify_content,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "space-between" => SpaceBetween,
        "space-around" => SpaceAround,
        "space-evenly" => SpaceEvenly,
    );

    impl_style_enum!(Overflow, "overflow", OverflowProperty, overflow,
        "visible" => Visible,
        "hidden" => Hidden,
    );
}

/// Impls for `bevy_text` [`Text`] component
mod text {
    use super::*;
    use crate::Defaults;

    #[derive(Default, Clone)]
    pub enum FontPath {
        #[default]
        Regular,
        Bold,
        Italic,
        BoldItalic,
        Custom(String),
    }

    /// Applies the `color` property on [`TextStyle::color`](`TextStyle`) field of all sections on matched [`Text`] components.
    #[derive(Default)]
    pub(crate) struct FontColorProperty;

    impl Property for FontColorProperty {
        type Item = Color;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("color")
        }

        fn affects_virtual_elements() -> bool {
            true
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
            values.color()
        }

        fn apply<'w>(
            cache: &Self::Item,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
            _entity: Entity,
        ) {
            components
                .sections
                .iter_mut()
                .for_each(|section| section.style.color = *cache);
        }
    }

    /// Applies the `font` property on [`TextStyle::font`](`TextStyle`) property of all sections on matched [`Text`] components.
    #[derive(Default)]
    pub(crate) struct FontProperty;

    impl Property for FontProperty {
        type Item = FontPath;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("font")
        }

        fn affects_virtual_elements() -> bool {
            true
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
            if let Ok(path) = values.string() {
                Ok(FontPath::Custom(path))
            } else if let Some(ident) = values.identifier() {
                match ident {
                    "regular" => Ok(FontPath::Regular),
                    "bold" => Ok(FontPath::Bold),
                    "italic" => Ok(FontPath::Italic),
                    "bold-italic" => Ok(FontPath::BoldItalic),
                    _ => Err(ElementsError::InvalidPropertyValue(
                        Self::name().to_string(),
                    )),
                }
            } else {
                Err(ElementsError::InvalidPropertyValue(
                    Self::name().to_string(),
                ))
            }
        }

        fn apply<'w>(
            cache: &Self::Item,
            mut components: QueryItem<Self::Components>,
            asset_server: &AssetServer,
            commands: &mut Commands,
            entity: Entity,
        ) {
            if let FontPath::Custom(path) = cache {
                components
                    .sections
                    .iter_mut()
                    .for_each(|section| section.style.font = asset_server.load(path));
            } else {
                let path = cache.clone();
                commands.add(move |world: &mut World| {
                    let defaults = world.resource::<Defaults>();
                    let font = match path {
                        FontPath::Regular => defaults.regular_font.clone(),
                        FontPath::Italic => defaults.italic_font.clone(),
                        FontPath::Bold => defaults.bold_font.clone(),
                        FontPath::BoldItalic => defaults.bold_italic_font.clone(),
                        _ => defaults.regular_font.clone(),
                    };
                    world
                        .entity_mut(entity)
                        .get_mut::<Text>()
                        .unwrap()
                        .sections
                        .iter_mut()
                        .for_each(|section| section.style.font = font.clone());
                });
            }
        }
    }

    /// Applies the `font-size` property on [`TextStyle::font_size`](`TextStyle`) property of all sections on matched [`Text`] components.
    #[derive(Default)]
    pub(crate) struct FontSizeProperty;

    impl Property for FontSizeProperty {
        type Item = f32;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("font-size")
        }

        fn affects_virtual_elements() -> bool {
            true
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
            values.f32()
        }

        fn apply<'w>(
            cache: &Self::Item,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
            _entity: Entity,
        ) {
            components
                .sections
                .iter_mut()
                .for_each(|section| section.style.font_size = *cache);
        }
    }

    /// Applies the `vertical-align` property on [`TextAlignment::vertical`](`TextAlignment`) property of matched [`Text`] components.
    #[derive(Default)]
    pub(crate) struct VerticalAlignProperty;

    impl Property for VerticalAlignProperty {
        // Using Option since Cache must impl Default, which VerticalAlign doesn't
        type Item = Option<VerticalAlign>;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("vertical-align")
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
            if let Some(ident) = values.identifier() {
                match ident {
                    "top" => return Ok(Some(VerticalAlign::Top)),
                    "center" => return Ok(Some(VerticalAlign::Center)),
                    "bottom" => return Ok(Some(VerticalAlign::Bottom)),
                    _ => (),
                }
            }
            Err(ElementsError::InvalidPropertyValue(
                Self::name().to_string(),
            ))
        }

        fn apply<'w>(
            cache: &Self::Item,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
            _entity: Entity,
        ) {
            components.alignment.vertical = cache.expect("Should always have a inner value");
        }
    }

    /// Applies the `text-align` property on [`TextAlignment::horizontal`](`TextAlignment`) property of matched [`Text`] components.
    #[derive(Default)]
    pub(crate) struct HorizontalAlignProperty;

    impl Property for HorizontalAlignProperty {
        // Using Option since Cache must impl Default, which HorizontalAlign doesn't
        type Item = Option<HorizontalAlign>;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("text-align")
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
            if let Some(ident) = values.identifier() {
                match ident {
                    "left" => return Ok(Some(HorizontalAlign::Left)),
                    "center" => return Ok(Some(HorizontalAlign::Center)),
                    "right" => return Ok(Some(HorizontalAlign::Right)),
                    _ => (),
                }
            }
            Err(ElementsError::InvalidPropertyValue(
                Self::name().to_string(),
            ))
        }

        fn apply<'w>(
            cache: &Self::Item,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
            _entity: Entity,
        ) {
            components.alignment.horizontal = cache.expect("Should always have a inner value");
        }
    }

    /// Apply a custom `text-content` which updates [`TextSection::value`](`TextSection`) of all sections on matched [`Text`] components
    #[derive(Default)]
    pub(crate) struct TextContentProperty;

    impl Property for TextContentProperty {
        type Item = String;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("text-content")
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
            values.string()
        }

        fn apply<'w>(
            cache: &Self::Item,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
            _entity: Entity,
        ) {
            components
                .sections
                .iter_mut()
                // TODO: Maybe change this so each line break is a new section
                .for_each(|section| section.value = cache.clone());
        }
    }
}

mod transform {
    use super::*;
    use bevy::prelude::Transform;

    #[derive(Default)]
    pub(crate) struct ScaleProperty;

    impl Property for ScaleProperty {
        type Item = f32;
        type Components = &'static mut Transform;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("scale")
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
            values.f32()
        }

        fn apply<'w>(
            cache: &Self::Item,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
            _entity: Entity,
        ) {
            components.scale = Vec3::splat(*cache);
        }
    }
}

/// Applies the `background-color` property on [`BackgroundColor`] component of matched entities.
#[derive(Default)]
pub(crate) struct BackgroundColorProperty;

impl Property for BackgroundColorProperty {
    type Item = Color;
    type Components = Entity;
    type Filters = With<BackgroundColor>;

    fn name() -> Tag {
        tag!("background-color")
    }

    fn parse<'a>(values: &StyleProperty) -> Result<Self::Item, ElementsError> {
        values.color()
    }

    fn apply<'w>(
        cache: &Self::Item,
        components: QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        commands: &mut Commands,
        _entity: Entity,
    ) {
        commands.entity(components).insert(BackgroundColor(*cache));
    }
}
