use crate::relations::bind::AsTransformer;
use crate::relations::bind::Prop;
use crate::relations::bind::TransformationError;
use crate::relations::bind::TransformationResult;
use crate::Transformers;
use bevy::prelude::*;

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

macro_rules! impl_color_channel_transformer {
    ($func: ident, $ch:ident, $setter:ident, $val:ident, $op:expr; $($args:ident,)*) => {
        pub fn $func<T: TransformableTo<f32>>(
            &self,
        ) -> fn(&T, Prop<Color>) -> TransformationResult {
            |source, mut color| {
                let $val = T::transform(source)?;
                let $val = $op;
                if color.$ch() != $val {
                    color.$setter($val);
                }
                Ok(())
            }
        }
    };
}

pub trait PropertySetter<P, V> {
    fn set(&self, property: &mut P, value: &V);
}

impl<F: Fn(&V, Prop<P>) -> TransformationResult, P, V> PropertySetter<P, V> for F {
    fn set(&self, mut prop: &mut P, value: &V) {
        let prop = Prop::new(&mut prop);
        if let Err(e) = self(value, prop) {
            error!("{e}")
        }
    }
}

fn t(color: &mut BackgroundColor) {
    Color::as_transformer().a().set(&mut color.0, &0.5)
}

// pub struct AssociatedColorTransformer

pub struct ColorTransformer;
impl ColorTransformer {
    impl_color_channel_transformer! { r, r, set_r, r, r.min(1.).max(0.); }
    impl_color_channel_transformer! { one_minus_r, r, set_r, r, (1.0 - r).min(1.).max(0.); }
    impl_color_channel_transformer! { g, g, set_g, g, g.min(1.).max(0.); }
    impl_color_channel_transformer! { one_minus_g, g, set_g, g, (1.0 - g).min(1.).max(0.); }
    impl_color_channel_transformer! { b, b, set_b, b, b.min(1.).max(0.); }
    impl_color_channel_transformer! { one_minus_b, b, set_b, b, (1.0 - b).min(1.).max(0.); }
    impl_color_channel_transformer! { a, a, set_a, a, a.min(1.).max(0.); }
    impl_color_channel_transformer! { one_minus_a, a, set_a, a, (1.0 - a).min(1.).max(0.); }

    // I'd like to leave here an idea of transformers with params. It is already implemented for
    // global transformers, but not possible for associated transformer yet.
    // I prefer to keep global/assoicated transformers api to be corresponding and silence this
    // implementation on purpose.
    // The calling a global transformer with args with current macro implementation looks like this:
    // from!(player, Health:valyue|color.clamp_r(0.2, 0.8))
    // Associatiated transformers call may looks exactly the same, but current implementation for
    // associated transformers can't handle such a feature
    //
    // impl_color_channel_transformer! { clamp_a, a, set_a, a, a.min(t).max(f); f, t, }
    // impl_color_channel_transformer! { clamp_one_minus_a, a, set_a, a, (1. - a).min(t).max(f); f, t, }
}

impl AsTransformer for Color {
    type Transformer = ColorTransformer;
    fn as_transformer() -> Self::Transformer {
        ColorTransformer
    }
}

pub trait ColorTransformerExtension {
    fn color() -> ColorTransformer {
        ColorTransformer
    }
}
impl ColorTransformerExtension for Transformers {}
