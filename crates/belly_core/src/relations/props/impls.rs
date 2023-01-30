use crate::impl_properties;
use bevy::prelude::*;

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
