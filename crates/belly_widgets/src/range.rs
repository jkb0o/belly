use super::common::*;
use belly_core::{build::*, impl_properties};
use belly_macro::*;
use bevy::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;

pub mod prelude {
    pub use super::LayoutMode;
    pub use super::Range;
    pub use super::RangeWidgetExtension;
}

pub(crate) struct RangePlugin;
impl Plugin for RangePlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<RangeWidget>();
        app.add_systems(Update, update_range_representation);
        app.add_systems(Update, configure_range_layout);
    }
}

#[widget]
#[styles = RANGE_STYLES]
/// Specifies the minimum value
#[param(minimum:f32 => Range:value|RangeValue.minimum)]
/// Specifies the maximum value
#[param(maximum:f32 => Range:value|RangeValue.maximum)]
/// Specifies absolute value in minimum..maximum range
#[param(value:f32 => Range:value|RangeValue.absolute)]
/// Specifies raltive value in 0..1 range
#[param(relative:f32 => Range:value|RangeValue.relative)]
/// <!-- @inline LayoutMode -->
#[param(mode:LayoutMode => Range:mode)]
fn range(ctx: &mut WidgetContext, rng: &mut Range) {
    let holder = rng.holder;
    let low = rng.low_span;
    let hight = rng.high_span;
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

ess_define! {
    RANGE_STYLES,

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
    /** @layout-aware */
    range .range-holder {
        width: 100%;
        height: 100%;
    }

    /** @layout-aware */
    range:horizontal .range-low-internals {
        height: 100%;
        width: undefined;
    }
    range:vertical .range-low-internals {
        height: undefined;
        width: 100%;
    }
    /** @layout-aware */
    range .range-high-internals {
        width: 100%;
        height: 100%;
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct RangeValue {
    minimum: f32,
    absolute: f32,
    maximum: f32,
}

impl Default for RangeValue {
    fn default() -> Self {
        RangeValue {
            minimum: 0.0,
            absolute: 0.0,
            maximum: 1.0,
        }
    }
}

impl RangeValue {
    pub fn absolute(&self) -> f32 {
        self.absolute
    }

    pub fn minimum(&self) -> f32 {
        self.minimum
    }

    pub fn maximum(&self) -> f32 {
        self.maximum
    }

    pub fn relative(&self) -> f32 {
        (self.absolute - self.minimum) / (self.maximum - self.minimum)
    }

    pub fn set_absolute(&mut self, value: f32) {
        self.absolute = value.min(self.maximum).max(self.minimum);
    }

    pub fn set_relative(&mut self, relative: f32) {
        let relative = relative.min(1.0).max(0.0);
        self.absolute = self.minimum + relative * (self.maximum - self.minimum);
    }

    pub fn set_minimum(&mut self, minimum: f32) {
        self.minimum = minimum.min(self.maximum);
        if self.absolute < self.minimum {
            self.absolute = self.minimum
        }
    }

    pub fn set_maximum(&mut self, maximum: f32) {
        self.maximum = maximum.max(self.minimum);
        if self.absolute > self.maximum {
            self.absolute = self.maximum;
        }
    }
}

impl_properties! { RangeValueProperties for RangeValue {
    absolute(set_absolute, absolute) => |v: f32| v.clone();
    minimum(set_minimum, minimum) => |v: f32| v.clone();
    maximum(set_maximum, maximum) => |v: f32| v.clone();
    relative(set_relative, relative) => |v: f32| v.clone();
}}

#[derive(Component)]
pub struct Range {
    pub value: RangeValue,
    pub mode: LayoutMode,

    pub holder: Entity,
    pub low_span: Entity,
    pub high_span: Entity,
}

impl FromWorldAndParams for Range {
    fn from_world_and_params(world: &mut World, params: &mut belly_core::eml::Params) -> Self {
        Range {
            value: RangeValue::default(),
            holder: world.spawn_empty().id(),
            low_span: world.spawn_empty().id(),
            high_span: world.spawn_empty().id(),
            mode: params.try_get("mode").unwrap_or_default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
/// Specifies the widget layout arrange.
/// <!-- @alter
/// - `verrtical`: arrange the widget vertically
/// - `horizontal`: arrange the widget horisontally
/// -->
pub enum LayoutMode {
    /// arrange items from top to bottom
    Vertical,
    #[default]
    /// arrange items from left to right
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
        let Ok(low) = nodes.get(low_span) else {
            continue;
        };
        let Ok(high) = nodes.get(high_span) else {
            continue;
        };
        let Ok(mut style) = styles.get_mut(low_span) else {
            continue;
        };
        let size = low.size() + high.size();
        let offset = size * range.value.relative();
        match range.mode {
            LayoutMode::Horizontal => style.min_width = Val::Px(offset.x),
            LayoutMode::Vertical => style.min_height = Val::Px(offset.y),
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
                    elements.set_state(entity, Tag::new("horizontal"), true);
                    elements.set_state(entity, Tag::new("vertical"), false);
                }
                LayoutMode::Vertical => {
                    elements.set_state(entity, Tag::new("horizontal"), false);
                    elements.set_state(entity, Tag::new("vertical"), true);
                }
            }
        }
        {
            let Ok(mut holder) = styles.get_mut(progress.holder) else {
                continue;
            };
            holder.flex_direction = match mode {
                LayoutMode::Horizontal => FlexDirection::Row,
                LayoutMode::Vertical => FlexDirection::ColumnReverse,
            }
        }
        {
            let Ok(mut low) = styles.get_mut(progress.low_span) else {
                continue;
            };
            match mode {
                LayoutMode::Horizontal => low.min_height = Val::Px(0.),
                LayoutMode::Vertical => low.min_width = Val::Px(0.),
            }
        }
    }
}
