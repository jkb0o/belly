use std::str::FromStr;

use crate::common::*;
use crate::input::button::*;
use bevy::prelude::*;
use bevy_elements_core::eml::build::FromWorldAndParam;
use bevy_elements_core::{relations::bind::Transformable, *};
use bevy_elements_macro::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
enum Label {
    GrabberInput,
}

pub(crate) struct SliderPlugin;
impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            process_grabber_system
                .after(input::Label::Signals)
                .label(Label::GrabberInput),
        );
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            value_from_input_system.after(Label::GrabberInput),
        );
        app.add_system(input_from_value_system);
        app.add_system(configure_slider_mode_system);
        app.register_widget::<Slider>();
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Range {
    value: f32,
    minimum: f32,
    maximum: f32,
}

impl FromWorldAndParam for Range {
    fn from_world_and_param(_world: &mut World, param: Variant) -> Self {
        let Variant::Params(mut params) = param else {
            return Range {
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
        Range {
            value,
            minimum,
            maximum,
        }
    }
}

impl Range {
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

impl Transformable for Range {
    type Transformer = RangeTransformer;
    fn transformer() -> Self::Transformer {
        RangeTransformer
    }
}

pub struct RangeTransformer;

macro_rules! impl_transform {
    ($method:ident, $setter:ident) => {
        pub fn $method<T: TransformableTo<f32>>(
            &self,
        ) -> fn(&T, Prop<Range>) -> TransformationResult {
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
impl RangeTransformer {
    impl_transform! { value, set_value }
    impl_transform! { minimum, set_minimum }
    impl_transform! { maximum, set_maximum }
    impl_transform! { relative, set_relative }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SliderMode {
    Vertical,
    Horizontal,
}

impl From<SliderMode> for Variant {
    fn from(m: SliderMode) -> Self {
        Variant::boxed(m)
    }
}

impl FromStr for SliderMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vertical" => Ok(SliderMode::Vertical),
            "horizontal" => Ok(SliderMode::Horizontal),
            s => Err(format!("Don't know how to parse '{s}' as SliderMode")),
        }
    }
}

impl TryFrom<Variant> for SliderMode {
    type Error = String;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        value.get_or_parse()
    }
}

impl FromWorldAndParam for SliderMode {
    fn from_world_and_param(_world: &mut World, param: Variant) -> Self {
        param.get_or(SliderMode::Horizontal)
    }
}

#[derive(Component, Widget)]
#[alias(slider)]
pub struct Slider {
    #[param(minimum: f32)]
    #[param(value: f32)]
    #[param(relative: f32)]
    #[param(maximum: f32)]
    pub range: Range,
    #[param]
    pub mode: SliderMode,

    configured_mode: Option<SliderMode>,
    holder: Entity,
    low_span: Entity,
    high_span: Entity,
    sliding: bool,
}

#[derive(Component)]
struct SliderGrabber {
    holder: Entity,
    low: Entity,
    slider: Entity,
}

#[derive(Component)]
struct SliderGrabberHolder {}

impl WidgetBuilder for Slider {
    fn setup(&mut self, ctx: &mut ElementContext) {
        let holder = self.holder;
        let low = self.low_span;
        let hight = self.high_span;
        let grabber = SliderGrabber {
            holder,
            low,
            slider: ctx.entity(),
        };
        ctx.render(eml! {
            <span c:slider>
                <span c:slider-back/>
                <span {holder} c:slider-holder s:flex-direction=managed()>
                    <span {low} c:slider-low-internals
                        s:min-height=managed()
                        s:min-width=managed()>

                        <span c:slider-low/>
                    </span>
                    <button with=grabber mode="instant" c:slider-grabber/>
                    <span {hight} c:slider-high-internals>
                        <span c:slider-high/>
                    </span>

                </span>
            </span>
        })
    }

    fn styles() -> &'static str {
        r##"
            slider:horizontal {
                padding: 5px 3px;
            }
            slider:vertical {
                padding: 3px 5px;
            }
            slider .slider-back {
                position-type: absolute;
                background-color: #ffffff;
            }
            slider:horizontal .slider-back {
                left: 0px;
                right: 0px;
                top: 10px;
                bottom: 9px;
            }
            slider:vertical .slider-back {
                left: 10px;
                right: 9px;
                top: 0px;
                bottom: 0px;
            }
            slider .slider-grabber {
                margin: 0px;
                min-width: 16px;
                min-height: 16px;
                width: 16px;
                height: 16px;
            }
            slider .slider-low {
                position-type: absolute;
                background-color: #4f4f4fdf;
            }
            slider:horizontal .slider-low {
                left: -2px;
                right: -1px;
                top: 6px;
                bottom: 5px;
            }
            slider:vertical .slider-low {
                top: -2px;
                bottom: -1px;
                left: 6px;
                right: 5px;
            }
            slider .slider-high {
                background-color: #bfbfbf;
                position-type: absolute;
            }
            slider:horizontal .slider-high {
                left: 1px;
                right: -2px;
                top: 6px;
                bottom: 5px;
            }
            slider:vertical .slider-high {
                top: 1px;
                bottom: -2px;
                left: 6px;
                right: 5px;
            }
            /* @layout-aware */
            slider .slider-holder {
                width: 100%;
                height: 100%;
            }
            
            /* @layout-aware */
            slider:horizontal .slider-low-internals {
                height: 100%;
                width: undefined;
            }
            slider:vertical .slider-low-internals {
                height: undefined;
                width: 100%;
            }
            /* @layout-aware */
            slider .slider-high-internals {
                width: 100%;
                height: 100%;
            }

        "##
    }
}

fn process_grabber_system(
    mut events: EventReader<PointerInput>,
    mut sliders: Query<&mut Slider>,
    grabbers: Query<(Entity, &SliderGrabber, &Node)>,
    mut styles: Query<&mut Style>,
    holders: Query<(&GlobalTransform, &Node)>,

    mut active_grabber: Local<Option<Entity>>,
    mut active_slider: Local<Option<Entity>>,
) {
    for ev in events.iter() {
        if active_grabber.is_some() && ev.drag_stop() {
            if let Ok(mut slider) = sliders.get_mut(active_slider.unwrap()) {
                slider.sliding = false;
            }
            *active_slider = None;
            *active_grabber = None;
        } else if ev.drag_start() && active_grabber.is_none() {
            let Some((entity, grabber, _)) = ev.entities
                .iter()
                .find_map(|e| grabbers.get(*e).ok())
                else { return };
            *active_grabber = Some(entity);
            *active_slider = Some(grabber.slider);
            if let Ok(mut slider) = sliders.get_mut(active_slider.unwrap()) {
                slider.sliding = true;
            }
        } else if active_grabber.is_some() && ev.dragging() {
            let entity = active_grabber.unwrap();
            let Ok(slider) = sliders.get(active_slider.unwrap()) else { continue };
            let Ok((_, grabber, gnode)) = grabbers.get(entity) else { continue };
            let Ok((htr, hnode)) = holders.get(grabber.holder) else { continue };
            let Ok(mut style) = styles.get_mut(grabber.low) else { continue };
            let grabber_offset = gnode.size() * 0.5;
            let pos = ev.pos - htr.translation().truncate() + hnode.size() * 0.5;
            let mut offset = (pos - grabber_offset).min(hnode.size() - gnode.size());
            offset.y = hnode.size().y - offset.y - gnode.size().y;
            offset.y = offset.y.min(hnode.size().y - gnode.size().y);
            match slider.mode {
                SliderMode::Horizontal => style.min_size.width = Val::Px(offset.x.max(0.)),
                SliderMode::Vertical => style.min_size.height = Val::Px(offset.y.max(0.)),
            }
        }
    }
}

fn value_from_input_system(mut sliders: Query<&mut Slider>, nodes: Query<&Node>) {
    for mut slider in sliders.iter_mut().filter(|s| s.sliding) {
        let Ok(low) = nodes.get(slider.low_span) else { continue };
        let Ok(high) = nodes.get(slider.high_span) else { continue };
        let relative = low.size() / (low.size() + high.size());
        match slider.mode {
            SliderMode::Horizontal => slider.range.set_relative(relative.x),
            SliderMode::Vertical => slider.range.set_relative(relative.y),
        }
    }
}

fn input_from_value_system(
    sliders: Query<&Slider, Or<(Changed<Slider>, Changed<Node>)>>,
    nodes: Query<&Node>,
    mut styles: Query<&mut Style>,
) {
    for slider in sliders.iter().filter(|s| !s.sliding) {
        let Ok(low) = nodes.get(slider.low_span) else { continue };
        let Ok(high) = nodes.get(slider.high_span) else { continue };
        let Ok(mut style) = styles.get_mut(slider.low_span) else { continue };
        let size = low.size() + high.size();
        let offset = size * slider.range.relative();
        match slider.mode {
            SliderMode::Horizontal => style.min_size.width = Val::Px(offset.x),
            SliderMode::Vertical => style.min_size.height = Val::Px(offset.y),
        }
    }
}

fn configure_slider_mode_system(
    mut elements: Elements,
    mut sliders: Query<(Entity, &mut Slider), Changed<Slider>>,
    mut styles: Query<&mut Style>,
) {
    for (entity, mut slider) in sliders.iter_mut() {
        if slider.configured_mode == Some(slider.mode) {
            continue;
        }
        slider.configured_mode = Some(slider.mode);
        {
            match slider.mode {
                SliderMode::Horizontal => {
                    elements.set_state(entity, "horizontal".as_tag(), true);
                    elements.set_state(entity, "vertical".as_tag(), false);
                }
                SliderMode::Vertical => {
                    elements.set_state(entity, "horizontal".as_tag(), false);
                    elements.set_state(entity, "vertical".as_tag(), true);
                }
            }
        }
        {
            let Ok(mut holder) = styles.get_mut(slider.holder) else { continue };
            holder.flex_direction = match slider.mode {
                SliderMode::Horizontal => FlexDirection::Row,
                SliderMode::Vertical => FlexDirection::ColumnReverse,
            }
        }
        {
            let Ok(mut low) = styles.get_mut(slider.low_span) else { continue };
            match slider.mode {
                SliderMode::Horizontal => low.min_size.height = Val::Undefined,
                SliderMode::Vertical => low.min_size.width = Val::Undefined,
            }
        }
    }
}
