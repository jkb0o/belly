use crate::TransformationResult;
use bevy::prelude::*;

use crate::relations::bind::{Prop, TransformationError};
use crate::Transformers;

pub trait TransformableTo<T: Clone> {
    fn transform(value: &Self) -> Result<T, TransformationError>;
}

impl<T: Clone + Sized> TransformableTo<T> for T {
    fn transform(value: &Self) -> Result<T, TransformationError> {
        Ok(value.clone())
    }
}

impl TransformableTo<f32> for &str {
    fn transform(value: &Self) -> Result<f32, TransformationError> {
        value.parse().map_err(TransformationError::from)
    }
}

impl TransformableTo<f32> for String {
    fn transform(value: &Self) -> Result<f32, TransformationError> {
        value.parse().map_err(TransformationError::from)
    }
}

pub trait ColorTransformerExtension {
    fn color() -> ColorTransformer {
        ColorTransformer
    }
}

impl ColorTransformerExtension for Transformers {}

macro_rules! impl_color_channel_transformer {
    ($func: ident, $ch:ident, $setter:ident, $val:ident, $op:expr; $($args:ident,)*) => {
        pub fn $func<T: TransformableTo<f32>>(
            &self,
            source: &T,
            mut color: Prop<Color>,
            $($args:f32,)*
        ) -> TransformationResult {
            let $val = T::transform(source)?;
            let $val = $op;
            if color.$ch() != $val {
                color.$setter($val);
            }
            Ok(())
        }
    };
}

trait Lerp {
    fn lerp(self, a: Self, b: Self) -> Self;
}

impl Lerp for f32 {
    fn lerp(self, a: Self, b: Self) -> Self {
        a + self * (b - a)
    }
}

pub struct ColorTransformer;
impl ColorTransformer {
    impl_color_channel_transformer! { r, r, set_r, r, r.min(1.).max(0.); }
    impl_color_channel_transformer! { one_minus_r, r, set_r, r, (1.0 - r).min(1.).max(0.); }
    impl_color_channel_transformer! { clamp_r, r, set_r, r, r.min(t).max(f); f, t, }
    impl_color_channel_transformer! { clamp_one_minus_r, r, set_r, r, (1. - r).min(t).max(f); f, t, }
    impl_color_channel_transformer! { lerp_r, r, set_r, r, r.lerp(f, t); f, t, }

    impl_color_channel_transformer! { g, g, set_g, g, g; }
    impl_color_channel_transformer! { g_minus_one, g, set_g, g, g - 1.0; }
    impl_color_channel_transformer! { one_minus_g, g, set_g, g, (1.0 - g).min(1.).max(0.); }
    impl_color_channel_transformer! { clamp_g, g, set_g, g, g.min(t).max(f); f, t, }
    impl_color_channel_transformer! { clamp_one_minus_g, g, set_g, g, (1. - g).min(t).max(f); f, t, }
    impl_color_channel_transformer! { lerp_g, g, set_g, g, g.lerp(f, t); f, t, }

    impl_color_channel_transformer! { b, b, set_b, b, b; }
    impl_color_channel_transformer! { one_minus_b, b, set_b, b, 1.0 - b; }
    impl_color_channel_transformer! { clamp_b, b, set_b, b, b.min(t).max(f); f, t, }
    impl_color_channel_transformer! { clamp_one_minus_b, b, set_b, b, (1. - b).min(t).max(f); f, t, }
    impl_color_channel_transformer! { lerp_b, b, set_b, b, b.lerp(f, t); f, t, }

    impl_color_channel_transformer! { a, a, set_a, a, a; }
    impl_color_channel_transformer! { one_minus_a, a, set_a, a, 1.0 - a; }
    impl_color_channel_transformer! { clamp_a, a, set_a, a, a.min(t).max(f); f, t, }
    impl_color_channel_transformer! { clamp_one_minus_a, a, set_a, a, (1. - a).min(t).max(f); f, t, }
    impl_color_channel_transformer! { lerp_a, a, set_a, a, a.lerp(f, t); f, t, }

    pub fn lerp<T: TransformableTo<f32>>(
        &self,
        source: &T,
        mut color: Prop<Color>,
        a: Color,
        b: Color,
    ) -> TransformationResult {
        let val = T::transform(source)?;
        let cr = val.lerp(a.r(), b.r());
        let cg = val.lerp(a.g(), b.g());
        let cb = val.lerp(a.b(), b.b());
        let ca = val.lerp(a.a(), b.a());
        let val = Color::rgba(cr, cg, cb, ca);
        if val != *color {
            *color = val;
        }
        Ok(())
    }
}
