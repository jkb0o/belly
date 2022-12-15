use crate::TransformationResult;
use bevy::prelude::*;
use tagstr::Tag;

use crate::Transformers;
pub trait TryIntoFloat {
    fn try_into_float(self) -> Result<f32, String>;
}

impl TryIntoFloat for f32 {
    fn try_into_float(self) -> Result<f32, String> {
        Ok(self)
    }
}

impl TryIntoFloat for f64 {
    fn try_into_float(self) -> Result<f32, String> {
        Ok(self as f32)
    }
}

impl TryIntoFloat for usize {
    fn try_into_float(self) -> Result<f32, String> {
        Ok(self as f32)
    }
}

impl TryIntoFloat for &str {
    fn try_into_float(self) -> Result<f32, String> {
        self.parse::<f32>().map_err(|e| e.to_string())
    }
}

impl TryIntoFloat for String {
    fn try_into_float(self) -> Result<f32, String> {
        self.as_str().try_into_float()
    }
}
impl TryIntoFloat for Tag {
    fn try_into_float(self) -> Result<f32, String> {
        self.as_str().try_into_float()
    }
}

pub trait ColorTransformerExtension {
    fn color() -> ColorTransformer {
        ColorTransformer
    }
}

impl ColorTransformerExtension for Transformers {}

macro_rules! impl_color_channel_transformer {
    ($func: ident, $ch:ident, $setter:ident, $val:ident, $op:expr) => {
        pub fn $func<T: TryIntoFloat + Clone>(
            &self,
            source: &T,
            color: &Color,
        ) -> TransformationResult<Color> {
            let mut color = color.clone();
            let $val = match source.clone().try_into_float() {
                Err(msg) => return TransformationResult::Invalid(msg),
                Ok(value) => value,
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

pub struct ColorTransformer;
impl ColorTransformer {
    impl_color_channel_transformer! { r, r, set_r, r, r }
    impl_color_channel_transformer! { one_minus_r, r, set_r, r, 1.0 - r }
    impl_color_channel_transformer! { g, g, set_g, g, g }
    impl_color_channel_transformer! { g_minus_one, g, set_g, g, g - 1.0 }
    impl_color_channel_transformer! { b, b, set_b, b, b }
    impl_color_channel_transformer! { one_minus_b, b, set_b, b, 1.0 - b }
    impl_color_channel_transformer! { a, a, set_a, a, a }
    impl_color_channel_transformer! { one_minus_a, a, set_a, a, 1.0 - a }
}
