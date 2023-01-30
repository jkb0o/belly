pub mod impls;

use super::bind::{BindableSource, BindableTarget, TransformationError};
use crate::relations::bind::TransformationResult;
use bevy::prelude::*;

pub fn try_transform<
    S: BindableSource,
    T: TryFrom<S, Error = E> + BindableTarget,
    E: Into<TransformationError>,
>(
    incoming: &S,
    mut current: Prop<T>,
) -> TransformationResult {
    let new_value = T::try_from(incoming.clone()).map_err(E::into)?;
    if new_value != *current {
        *current = new_value;
    }
    Ok(())
}

pub struct PropertyDescriptor<'a, 'c, C: Component, T> {
    changed: bool,
    component: &'a mut Mut<'c, C>,
    ref_getter: for<'b> fn(&'b Mut<C>) -> &'b T,
    mut_getter: for<'b> fn(&'b mut Mut<C>) -> &'b mut T,
}

impl<'a, 'c, C: Component, T> PropertyDescriptor<'a, 'c, C, T> {
    pub fn new(
        component: &'a mut Mut<'c, C>,
        ref_getter: for<'b> fn(&'b Mut<C>) -> &'b T,
        mut_getter: for<'b> fn(&'b mut Mut<C>) -> &'b mut T,
    ) -> Self {
        Self {
            changed: false,
            component,
            ref_getter,
            mut_getter,
        }
    }
    pub fn as_prop(&mut self) -> Prop<T> {
        Prop(self)
    }

    pub fn changed(&self) -> bool {
        self.changed
    }
}

impl<'a, 'c, C: Component, T> AsRef<T> for PropertyDescriptor<'a, 'c, C, T> {
    fn as_ref(&self) -> &T {
        (self.ref_getter)(&self.component)
    }
}

impl<'a, 'c, C: Component, T> AsMut<T> for PropertyDescriptor<'a, 'c, C, T> {
    fn as_mut(&mut self) -> &mut T {
        self.changed = true;
        (self.mut_getter)(&mut self.component)
    }
}

pub trait PropertyProtocol<T> {
    fn as_ref(&self) -> &T;
    fn as_mut(&mut self) -> &mut T;
}
impl<'a, 'c, C: Component, T> PropertyProtocol<T> for PropertyDescriptor<'a, 'c, C, T> {
    fn as_ref(&self) -> &T {
        (self.ref_getter)(&self.component)
    }

    fn as_mut(&mut self) -> &mut T {
        self.changed = true;
        (self.mut_getter)(&mut self.component)
    }
}
impl<T> PropertyProtocol<T> for T {
    fn as_mut(&mut self) -> &mut T {
        self
    }
    fn as_ref(&self) -> &T {
        self
    }
}

// impl<T, X: AsRef<T> + AsMut<T>> PropertyProtocol<T> for X {}

pub struct Prop<'a, T>(&'a mut dyn PropertyProtocol<T>);

impl<'a, T> Prop<'a, T> {
    pub fn new<P: PropertyProtocol<T>>(value: &'a mut P) -> Prop<'a, T> {
        Prop(value)
    }
}

impl<'a, T> std::ops::Deref for Prop<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        PropertyProtocol::<T>::as_ref(self.0)
    }
}

impl<'a, T> std::ops::DerefMut for Prop<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl<'a, T> From<&'a mut T> for Prop<'a, T> {
    fn from(value: &'a mut T) -> Self {
        Prop::new(value)
    }
}

#[derive(Clone, Copy)]
pub struct SetGet<P, V> {
    set_func: fn(&V, Prop<P>) -> TransformationResult,
    get_func: for<'a> fn(Prop<'a, P>) -> V,
}

impl<P, V> SetGet<P, V> {
    pub fn new(
        set: fn(&V, Prop<P>) -> TransformationResult,
        get: fn(Prop<P>) -> V,
    ) -> SetGet<P, V> {
        SetGet {
            set_func: set,
            get_func: get,
        }
    }
    pub fn set<'a>(&self, prop: Prop<'a, P>, value: &V) {
        if let Err(e) = (self.set_func)(value, prop) {
            error!("Error setting property: {e}")
        }
    }
    pub fn get<'a>(&self, prop: Prop<'a, P>) -> V {
        (self.get_func)(prop.into())
    }
    pub fn as_transformer(&self) -> fn(&V, Prop<P>) -> TransformationResult {
        self.set_func
    }
    pub fn getter(&self) -> for<'a> fn(Prop<'a, P>) -> V {
        self.get_func
    }
}

#[macro_export]
macro_rules! impl_properties {
    (@method $cls:ty, $prop:ident, $setter:ident, $getter:ident, $var:ident, $itemty:ty, $expr:expr) => {
        pub fn $prop(&self) -> $crate::relations::props::SetGet<$cls, $itemty> {
            fn set($var: &$itemty, mut prop: $crate::relations::props::Prop<$cls>) -> $crate::relations::bind::TransformationResult {
                let $var = $expr;
                if $var != prop.$getter() {
                    prop.$setter($var);
                }
                Ok(())
            }
            fn get(prop: $crate::relations::props::Prop<$cls>) -> $itemty {
                let $var = prop.$getter();
                let $var = $expr;
                $var
            }
            $crate::relations::props::SetGet::new(set, get)
        }

    };
    ($struct:ident for $cls:ty { $($prop:ident($setter:ident, $getter:ident) => |$var:ident: $itemty:ty| $expr:expr;)+ }) => {
        pub struct $struct;
        impl $crate::relations::props::GetProperties for $cls {
            type Item = $struct;
            fn get_properties() -> &'static Self::Item {
                &$struct
            }
        }
        impl $struct {
            $(impl_properties!{ @method $cls, $prop, $setter, $getter, $var, $itemty, $expr })+
        }
    };
    ($struct:ident for $cls:ty as $ext:ident { $($prop:ident($setter:ident, $getter:ident) => |$var:ident: $itemty:ty| $expr:expr;)+ }) => {
        pub struct $struct;
        impl $struct {
            $(impl_properties!{ @method $cls, $prop, $setter, $getter, $var, $itemty, $expr })+
        }
        pub trait $ext {
            fn get_properties() -> &'static $struct {
                &$struct
            }
        }
        impl $ext for $cls { }
    };
}

pub trait GetProperties {
    type Item;
    fn get_properties() -> &'static Self::Item;
}
