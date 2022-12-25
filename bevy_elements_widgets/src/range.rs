use std::str::FromStr;

use super::common::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_elements_core::{eml::build::FromWorldAndParam, relations::bind::Transformable, *};
use bevy_elements_macro::*;

pub(crate) struct RangePlugin;
impl Plugin for RangePlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<Range>();
        app.add_system(update_range_representation);
        app.add_system(configure_range_layout);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct LimitedValue {
    value: f32,
    minimum: f32,
    maximum: f32,
}

impl FromWorldAndParam for LimitedValue {
    fn from_world_and_param(_world: &mut World, param: Variant) -> Self {
        let Variant::Params(mut params) = param else {
            return LimitedValue {
                minimum: 0.0,
                value: 0.0,
                maximum: 1.0,
            }
        };
        let minimum = params.try_get::<f32>("minimum");
        let value = params.try_get::<f32>("value");
        let maximum = params.try_get::<f32>("maximum");
        let (minimum, value, maximum) = match (minimum, value, maximum) {
            (Some(min), Some(val), Some(max)) => {
                (min.min(max), val.max(min).min(max), max.max(min))
            }
            (None, Some(val), Some(max)) => (0.0f32.min(val).min(max), val.min(max), max),
            (Some(min), None, Some(max)) => (min.min(max), 0.0f32.min(max).max(min), max.max(min)),
            (Some(min), Some(val), None) => (min, val.max(min), 1.0f32.max(min).max(val)),
            (Some(min), None, None) => (min, 0.0f32.max(min), 1.0f32.max(min)),
            (None, Some(val), None) => (0.0f32.min(val), val, 1.0f32.max(val)),
            (None, None, Some(max)) => (0.0f32.min(max), 0.0f32.min(max), max),
            (None, None, None) => (0.0, 0.0, 1.0),
        };
        LimitedValue {
            value,
            minimum,
            maximum,
        }
    }
}

impl LimitedValue {
    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn minimum(&self) -> f32 {
        self.minimum
    }

    pub fn maximum(&self) -> f32 {
        self.maximum
    }

    pub fn relative(&self) -> f32 {
        (self.value - self.minimum) / (self.maximum - self.minimum)
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.min(self.maximum).max(self.minimum);
    }

    pub fn set_relative(&mut self, relative: f32) {
        let relative = relative.min(1.0).max(0.0);
        self.value = self.minimum + relative * (self.maximum - self.minimum);
    }

    pub fn set_minimum(&mut self, minimum: f32) {
        self.minimum = minimum.min(self.maximum);
        if self.value < self.minimum {
            self.value = self.minimum
        }
    }

    pub fn set_maximum(&mut self, maximum: f32) {
        self.maximum = maximum.max(self.minimum);
        if self.value > self.maximum {
            self.value = self.maximum;
        }
    }
}

impl Transformable for LimitedValue {
    type Transformer = LimitedValueTransformer;
    fn transformer() -> Self::Transformer {
        LimitedValueTransformer
    }
}

pub struct LimitedValueTransformer;

macro_rules! impl_transform {
    ($method:ident, $setter:ident) => {
        pub fn $method<T: TransformableTo<f32>>(
            &self,
        ) -> fn(&T, Prop<LimitedValue>) -> TransformationResult {
            |source, mut range| {
                let val = T::transform(source)?;
                if val != range.$method() {
                    range.$setter(val);
                }
                return Ok(());
            }
        }
    };
}
impl LimitedValueTransformer {
    impl_transform! { value, set_value }
    impl_transform! { minimum, set_minimum }
    impl_transform! { maximum, set_maximum }
    impl_transform! { relative, set_relative }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LayoutMode {
    Vertical,
    Horizontal,
}

impl From<LayoutMode> for Variant {
    fn from(m: LayoutMode) -> Self {
        Variant::boxed(m)
    }
}

impl FromStr for LayoutMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vertical" => Ok(LayoutMode::Vertical),
            "horizontal" => Ok(LayoutMode::Horizontal),
            s => Err(format!("Don't know how to parse '{s}' as LayoutMode")),
        }
    }
}

impl TryFrom<Variant> for LayoutMode {
    type Error = String;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        value.get_or_parse()
    }
}

impl FromWorldAndParam for LayoutMode {
    fn from_world_and_param(_world: &mut World, param: Variant) -> Self {
        param.get_or(LayoutMode::Horizontal)
    }
}

#[derive(Component, Widget)]
#[alias(range)]
pub struct Range {
    #[param(minimum: f32)]
    #[param(value: f32)]
    #[param(relative: f32)]
    #[param(maximum: f32)]
    pub value: LimitedValue,

    #[param]
    pub mode: LayoutMode,

    pub holder: Entity,
    pub low_span: Entity,
    pub high_span: Entity,
}

impl WidgetBuilder for Range {
    fn setup(&mut self, ctx: &mut ElementContext) {
        info!("range seup");
        let holder = self.holder;
        let low = self.low_span;
        let hight = self.high_span;
        ctx.render(eml! {
            <span c:range>
                <span c:range-back/>
                <span {holder} c:range-holder s:flex-direction=managed()>
                    <span {low} c:range-low-internals
                        s:min-height=managed()
                        s:min-width=managed()>
                        <span c:range-low/>
                    </span>
                    <slot define="separator"/>
                    <span {hight} c:range-high-internals>
                        <span c:range-high/>
                    </span>
                </span>
            </span>
        })
    }
    fn styles() -> &'static str {
        r#"
            range:horizontal {
                padding: 5px 3px;
            }
            range:vertical {
                padding: 3px 5px;
            }
            range .range-back {
                position-type: absolute;
                background-color: #ffffff;
            }
            range:horizontal .range-back {
                left: 0px;
                right: 0px;
                top: 10px;
                bottom: 9px;
            }
            range:vertical .range-back {
                left: 10px;
                right: 9px;
                top: 0px;
                bottom: 0px;
            }
            range .range-low {
                position-type: absolute;
                background-color: #4f4f4fdf;
            }
            range:horizontal .range-low {
                left: -2px;
                right: -1px;
                top: 6px;
                bottom: 5px;
            }
            range:vertical .range-low {
                top: -2px;
                bottom: -1px;
                left: 6px;
                right: 5px;
            }
            range .range-high {
                background-color: #bfbfbf;
                position-type: absolute;
            }
            range:horizontal .range-high {
                left: 1px;
                right: -2px;
                top: 6px;
                bottom: 5px;
            }
            range:vertical .range-high {
                top: 1px;
                bottom: -2px;
                left: 6px;
                right: 5px;
            }
            /* @layout-aware */
            range .range-holder {
                width: 100%;
                height: 100%;
            }
            
            /* @layout-aware */
            range:horizontal .range-low-internals {
                height: 100%;
                width: undefined;
            }
            range:vertical .range-low-internals {
                height: undefined;
                width: 100%;
            }
            /* @layout-aware */
            range .range-high-internals {
                width: 100%;
                height: 100%;
            }
        "#
    }
}

pub fn update_range_representation(
    ranges: Query<&Range, Or<(Changed<Range>, Changed<Node>)>>,
    nodes: Query<&Node>,
    mut styles: Query<&mut Style>,
) {
    for range in ranges.iter()
    // .filter(|s| !s.progress_updating_locked())
    {
        let low_span = range.low_span;
        let high_span = range.high_span;
        let Ok(low) = nodes.get(low_span) else { continue };
        let Ok(high) = nodes.get(high_span) else { continue };
        let Ok(mut style) = styles.get_mut(low_span) else { continue };
        let size = low.size() + high.size();
        let offset = size * range.value.relative();
        match range.mode {
            LayoutMode::Horizontal => style.min_size.width = Val::Px(offset.x),
            LayoutMode::Vertical => style.min_size.height = Val::Px(offset.y),
        }
    }
}

pub fn configure_range_layout(
    mut elements: Elements,
    progres_components: Query<(Entity, &Range), Changed<Range>>,
    mut styles: Query<&mut Style>,
    mut configured_modes: Local<HashMap<Entity, LayoutMode>>,
) {
    for (entity, progress) in progres_components.iter() {
        let mode = progress.mode;
        if configured_modes.get(&entity) == Some(&mode) {
            continue;
        }
        configured_modes.insert(entity, mode);
        {
            match mode {
                LayoutMode::Horizontal => {
                    elements.set_state(entity, "horizontal".as_tag(), true);
                    elements.set_state(entity, "vertical".as_tag(), false);
                }
                LayoutMode::Vertical => {
                    elements.set_state(entity, "horizontal".as_tag(), false);
                    elements.set_state(entity, "vertical".as_tag(), true);
                }
            }
        }
        {
            let Ok(mut holder) = styles.get_mut(progress.holder) else { continue };
            holder.flex_direction = match mode {
                LayoutMode::Horizontal => FlexDirection::Row,
                LayoutMode::Vertical => FlexDirection::ColumnReverse,
            }
        }
        {
            let Ok(mut low) = styles.get_mut(progress.low_span) else { continue };
            match mode {
                LayoutMode::Horizontal => low.min_size.height = Val::Undefined,
                LayoutMode::Vertical => low.min_size.width = Val::Undefined,
            }
        }
    }
}
