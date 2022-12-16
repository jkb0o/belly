use crate::TransformationResult;
use bevy::prelude::*;
use tagstr::Tag;

use crate::Transformers;

pub struct Float(f32);

macro_rules! impl_try_into_float {
    (@impl $t:ty) => {
        impl TryFrom<$t> for Float {
            type Error = String;
            fn try_from(value: $t) -> Result<Float, Self::Error> {
                Ok(Float(value as f32))
            }
        }
    };
    ( $t:ty, $($rest:ty,)* ) => {
        impl_try_into_float! { @impl $t }
        impl_try_into_float!{ $($rest,)* }
    };
    ( $t:ty ) => {
        impl_try_into_float! { @impl $t }
    };
    ( ) => { }
}

impl_try_into_float! {
    f32, f64,
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
}

impl TryFrom<&str> for Float {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let result: Result<f32, _> = value.parse();
        match result {
            Ok(f) => Ok(Float(f)),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl TryFrom<String> for Float {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Float::try_from(value.as_str())
    }
}

impl TryFrom<Tag> for Float {
    type Error = String;
    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        Float::try_from(value.as_str())
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
        pub fn $func<T: TryInto<Float, Error = E> + Clone, E: Into<String>>(
            &self,
            source: &T,
            color: &Color,
            $($args:f32,)*
        ) -> TransformationResult<Color> {
            let mut color = color.clone();
            let $val = match T::try_into(source.clone()) {
                Err(msg) => return TransformationResult::Invalid(msg.into()),
                Ok(value) => value.0,
            };
            let $val = $op;
            if color.$ch() == $val {
                TransformationResult::Unchanged
            } else {
                color.$setter($val);
                TransformationResult::Changed(color)
            }
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

    pub fn lerp<T: TryInto<Float, Error = E> + Clone, E: Into<String>>(
        &self,
        source: &T,
        color: &Color,
        a: Color,
        b: Color,
    ) -> TransformationResult<Color> {
        let color = color.clone();
        let val = match T::try_into(source.clone()) {
            Err(msg) => return TransformationResult::Invalid(msg.into()),
            Ok(value) => value.0,
        };
        let cr = val.lerp(a.r(), b.r());
        let cg = val.lerp(a.g(), b.g());
        let cb = val.lerp(a.b(), b.b());
        let ca = val.lerp(a.a(), b.a());
        let val = Color::rgba(cr, cg, cb, ca);
        if color == val {
            TransformationResult::Unchanged
        } else {
            TransformationResult::Changed(color)
        }
    }
}
