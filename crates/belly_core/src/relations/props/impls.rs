use std::marker::PhantomData;

use crate::{
    build::{Prop, TransformationResult},
    impl_properties,
    relations::bind::{BindableSource, BindableTarget},
};
use bevy::prelude::*;

use super::{GetProperties, SetGet};

impl_properties! { ColorProperties for Color {
    r(set_r, r) => |v: f32| v.min(1.).max(0.);
    g(set_g, g) => |v: f32| v.min(1.).max(0.);
    b(set_b, b) => |v: f32| v.min(1.).max(0.);
    a(set_a, a) => |v: f32| v.min(1.).max(0.);
    one_minus_r(set_r, r) => |v: f32| (1.0 - v).min(1.).max(0.);
    one_minus_g(set_g, g) => |v: f32| (1.0 - v).min(1.).max(0.);
    one_minus_b(set_b, b) => |v: f32| (1.0 - v).min(1.).max(0.);
    one_minus_a(set_a, a) => |v: f32| (1.0 - v).min(1.).max(0.);
}}

pub struct OptionProperties<T>(PhantomData<T>);
fn set_some<T: BindableSource + BindableTarget>(
    val: &T,
    mut prop: Prop<Option<T>>,
) -> TransformationResult {
    if Some(val) != prop.as_ref() {
        *prop = Some(val.clone());
    }
    Ok(())
}
fn get_some<T: BindableSource + BindableTarget>(prop: Prop<Option<T>>) -> T {
    if let Some(value) = prop.as_ref() {
        return value.clone();
    } else {
        panic!("Can't use OptionProperties.some to fetch empty value");
    }
}
impl<T: BindableSource + BindableTarget> OptionProperties<T> {
    pub fn some(&self) -> SetGet<Option<T>, T> {
        SetGet::new(set_some, get_some)
    }
}
impl<T> GetProperties for Option<T> {
    type Item = OptionProperties<T>;
    fn get_properties() -> &'static Self::Item {
        &OptionProperties(PhantomData)
    }
}
