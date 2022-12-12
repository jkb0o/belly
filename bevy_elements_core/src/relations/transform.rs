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
    ($ch:ident, $setter:ident) => {
        pub fn $ch<T: TryIntoFloat + Clone>(
            &self,
            alpha: &T,
            color: &Color,
        ) -> TransformationResult<Color> {
            let mut color = color.clone();
            let channel = match alpha.clone().try_into_float() {
                Err(msg) => return TransformationResult::Invalid(msg),
                Ok(value) => value,
            };
            if color.$ch() == channel {
                TransformationResult::Unchanged
            } else {
                color.$setter(channel);
                TransformationResult::Changed(color)
            }
        }
    };
}

pub struct ColorTransformer;
impl ColorTransformer {
    impl_color_channel_transformer! { r, set_r }
    impl_color_channel_transformer! { g, set_g }
    impl_color_channel_transformer! { b, set_b }
    impl_color_channel_transformer! { a, set_a }
}
