use bevy::{ecs::query::QueryItem, prelude::*};

use crate::ElementsError;
use tagstr::*;

use super::{Property, StyleProperty};

pub(crate) use style::*;
pub(crate) use text::*;

/// Impls for `bevy_ui` [`Style`] component
mod style {
    use super::*;
    /// Implements a new property for [`Style`] component which expects a rect value.
    macro_rules! impl_style_rect {
        ($name:expr, $struct:ident, $style_prop:ident$(.$style_field:ident)*) => {
            #[doc = "Applies the `"]
            #[doc = $name]
            #[doc = "` property on [Style::"]
            #[doc = stringify!($style_prop)]
            $(#[doc = concat!("::",stringify!($style_field))])*
            #[doc = "](`Style`) field of all sections on matched [`Style`] components."]
            #[derive(Default)]
            pub(crate) struct $struct;

            impl Property for $struct {
                type Cache = UiRect;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> Tag {
                    tag!($name)
                }

                fn parse<'a>(values: &StyleProperty) -> Result<Self::Cache, ElementsError> {
                    if let Some(val) = values.rect() {
                        Ok(val)
                    } else {
                        Err(ElementsError::InvalidPropertyValue(Self::name().to_string()))
                    }
                }

                fn apply<'w>(
                    cache: &Self::Cache,
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

    impl_style_rect!("margin", MarginProperty, margin);
    impl_style_rect!("padding", PaddingProperty, padding);
    impl_style_rect!("border", BorderProperty, border);

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
                type Cache = $cache;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> Tag {
                    tag!($name)
                }

                fn parse<'a>(values: &StyleProperty) -> Result<Self::Cache, ElementsError> {
                    if let Some(val) = values.$parse_func() {
                        Ok(val)
                    } else {
                        Err(ElementsError::InvalidPropertyValue(values.to_string()))
                    }
                }

                fn apply<'w>(
                    cache: &Self::Cache,
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
    impl_style_single_value!("left", LeftProperty, Val, val, position.left);
    impl_style_single_value!("right", RightProperty, Val, val, position.right);
    impl_style_single_value!("top", TopProperty, Val, val, position.top);
    impl_style_single_value!("bottom", BottomProperty, Val, val, position.bottom);

    impl_style_single_value!("margin-left", MarginLeftProperty, Val, val, margin.left);
    impl_style_single_value!("margin-right", MarginRightProperty, Val, val, margin.right);
    impl_style_single_value!("padding-left", PaddingLeftProperty, Val, val, padding.left);

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
                type Cache = $cache;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> Tag {
                    tag!($name)
                }

                fn parse<'a>(values: &StyleProperty) -> Result<Self::Cache, ElementsError> {
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
                    cache: &Self::Cache,
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
        type Cache = Color;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("color")
        }

        fn affects_virtual_elements() -> bool {
            true
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Cache, ElementsError> {
            if let Some(color) = values.color() {
                Ok(color)
            } else {
                Err(ElementsError::InvalidPropertyValue(
                    Self::name().to_string(),
                ))
            }
        }

        fn apply<'w>(
            cache: &Self::Cache,
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
        type Cache = FontPath;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("font")
        }

        fn affects_virtual_elements() -> bool {
            true
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Cache, ElementsError> {
            if let Some(path) = values.string() {
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
            cache: &Self::Cache,
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
        type Cache = f32;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("font-size")
        }

        fn affects_virtual_elements() -> bool {
            true
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Cache, ElementsError> {
            if let Some(size) = values.f32() {
                Ok(size)
            } else {
                Err(ElementsError::InvalidPropertyValue(
                    Self::name().to_string(),
                ))
            }
        }

        fn apply<'w>(
            cache: &Self::Cache,
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
        type Cache = Option<VerticalAlign>;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("vertical-align")
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Cache, ElementsError> {
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
            cache: &Self::Cache,
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
        type Cache = Option<HorizontalAlign>;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("text-align")
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Cache, ElementsError> {
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
            cache: &Self::Cache,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
            _entity: Entity,
        ) {
            components.alignment.horizontal = cache.expect("Should always have a inner value");
            info!("applying aligment = {:?}", components.alignment.horizontal);
        }
    }

    /// Apply a custom `text-content` which updates [`TextSection::value`](`TextSection`) of all sections on matched [`Text`] components
    #[derive(Default)]
    pub(crate) struct TextContentProperty;

    impl Property for TextContentProperty {
        type Cache = String;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> Tag {
            tag!("text-content")
        }

        fn parse<'a>(values: &StyleProperty) -> Result<Self::Cache, ElementsError> {
            if let Some(content) = values.string() {
                Ok(content)
            } else {
                Err(ElementsError::InvalidPropertyValue(
                    Self::name().to_string(),
                ))
            }
        }

        fn apply<'w>(
            cache: &Self::Cache,
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

/// Applies the `background-color` property on [`BackgroundColor`] component of matched entities.
#[derive(Default)]
pub(crate) struct BackgroundColorProperty;

impl Property for BackgroundColorProperty {
    type Cache = Color;
    type Components = Entity;
    type Filters = With<BackgroundColor>;

    fn name() -> Tag {
        tag!("background-color")
    }

    fn parse<'a>(values: &StyleProperty) -> Result<Self::Cache, ElementsError> {
        if let Some(color) = values.color() {
            Ok(color)
        } else {
            Err(ElementsError::InvalidPropertyValue(
                Self::name().to_string(),
            ))
        }
    }

    fn apply<'w>(
        cache: &Self::Cache,
        components: QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        commands: &mut Commands,
        _entity: Entity,
    ) {
        commands.entity(components).insert(BackgroundColor(*cache));
    }
}
