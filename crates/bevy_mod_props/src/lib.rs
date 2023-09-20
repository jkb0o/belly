use std::{rc::Rc, any::type_name, marker::PhantomData};


use bevy::prelude::*;

mod impls;

pub enum Write<'o, T> {
    Set(Rc<dyn Fn(T) + 'o>),
    Mut(Rc<dyn Fn() -> &'o mut T + 'o>)
}

impl<'o, T> Clone for Write<'o, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Set(s) => Self::Set(s.clone()),
            Self::Mut(m) => Self::Mut(m.clone()),
        }
    }
}

impl<'o, T> Write<'o, T> {
    pub fn setter<F: Fn(T) + 'o>(setter: F) -> Self {
        Self::Set(Rc::new(setter))
    }
    pub fn mutator<F: Fn() -> &'o mut T + 'o>(mutator: F) -> Self {
        Self::Mut(Rc::new(mutator))
    }
}



impl<'o, T: Reflect + FromReflect> Write<'o, T> {
    pub fn untyped(&self, reflected: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        if let Some(value) = T::from_reflect(reflected.as_ref()) {
            match self {
                Write::Mut(get_mut) => *get_mut() = value,
                Write::Set(set_val) => set_val(value)
            };
            Ok(())
        } else {
            Err(reflected)
        }
    }
}

pub enum Read<'o, T> {
    Ref(Rc<dyn Fn() -> &'o T + 'o>),
    Val(Rc<dyn Fn() -> T + 'o>),
}

impl<'o, T> Read<'o, T> {
    pub fn value<F: Fn() -> T + 'o>(read: F) -> Self {
        Self::Val(Rc::new(read))
    }
    pub fn reference<F: Fn() -> &'o T + 'o>(read: F) -> Self {
        Self::Ref(Rc::new(read))
    }
}

impl<'o, T> Clone for Read<'o, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Ref(r) => Self::Ref(r.clone()),
            Self::Val(v) => Self::Val(v.clone()),
        }
    }
}

impl <'o, T: Reflect> Read<'o, T> {
    pub fn untyped(&self) -> Box<dyn Reflect> {
        match self {
            Read::Ref(get_ref) => get_ref().as_reflect().clone_value(),
            Read::Val(get_val) => get_val().as_reflect().clone_value(),
        }
    }
}

pub struct Prop<'w, 'o, T, R> {
    read: Read<'o, T>,
    write: Write<'o, T>,
    marker: PhantomData<&'w R>
}

impl<'w, 'o, T, R> Prop<'w, 'o, T, R>{
    pub fn new(read: Read<'o, T>, write: Write<'o, T>) -> Self {
        Prop { read, write, marker: PhantomData }
    }
}

pub struct Imutable;
pub struct Mutable;

impl<'w, 'o, T: Component, R: 'static> Prop<'w, 'o, T, R> {
    pub fn from_ref(component: &'o T) -> Prop<'w, 'o, T, R> {
        Prop {
            read: Read::Ref(Rc::new(|| component)),
            write: Write::Mut(Rc::new(|| panic!("Attempt to write readonly property"))),
            marker: PhantomData
        }
    }
}

impl<'w, 'o, T: Component> Prop<'w, 'o, T, Mutable> {

    pub fn from_mut(component: &'o mut Mut<'w, T>) -> Prop<'w, 'o, T, Mutable> {
        let ptr: *mut Mut<'w, T> = component;
        let read = ptr.clone();
        let write = ptr.clone();
        
        Prop {
            read: Read::Ref(Rc::new(move || unsafe {
                read.as_ref().unwrap().as_ref()
            })),
            write: Write::Mut(Rc::new(move || unsafe {
                write.as_mut().unwrap().as_mut()
            })),
            marker: PhantomData
        }
    }
}

pub mod usage {
    pub use super::*;
    pub trait ImutableProp<'w, 'o, T, R> {
        fn reader(&self) -> Read<'o, T>;
        fn writer(&self) -> Write<'o, T>;
        fn with<F: FnMut(&T)>(&self, getter: F);
        fn get(&self) -> T where T: Clone;
    }
    pub trait MutableProp<'w, 'o, T, R> {
        fn set(&mut self, value: T);
        fn update(&mut self, value: T) where T: PartialEq;
    }
    impl<'w, 'o, T, R> ImutableProp<'w, 'o, T, R> for Prop<'w, 'o, T, R> {
        fn reader(&self) -> Read<'o, T> {
            self.read.clone()
        }

        fn writer(&self) -> Write<'o, T> {
            self.write.clone()
        }
        
        fn with<F: FnMut(&T)>(&self, mut getter: F) {
            match &self.read {
                Read::Ref(get_ref) => getter(get_ref()),
                Read::Val(get_value) => getter(&get_value())
            }
        }
        
        fn get(&self) -> T where T: Clone {
            match &self.read {
                Read::Ref(get_ref) => get_ref().clone(),
                Read::Val(get_val) => get_val()
            }
        }

    }
    impl<'w, 'o, T> MutableProp<'w, 'o, T, Mutable> for Prop<'w, 'o, T, Mutable> {
        fn set(&mut self, value: T) {
            match &self.write {
                Write::Mut(target) => *target() = value,
                Write::Set(setter) => setter(value)
            }
        }
        fn update(&mut self, value: T) where T: PartialEq {
            let mut changed = false;
            self.with(|v| if v != &value {
                changed = true;
            });
            if changed {
                self.set(value)
            }
        }

    }

    pub trait PropPath<'s> {
        fn parse_token(self) -> (PathToken<'s>, Self);
    }
    impl<'s> PropPath<'s> for &'s str {
        fn parse_token(self) -> (PathToken<'s>, Self) {
            if self.starts_with("[") {
                if let Some(closing_bracket_pos) = self.find(']') {
                    let idx = &self[1..closing_bracket_pos];
                    if let Ok(idx) = idx.parse() {
                        return (PathToken::Index(idx), &self[closing_bracket_pos+1..])
                    } else {
                        return (PathToken::Invalid, "");
                    }
                } else {
                    return (PathToken::Invalid, "");
                }
            }
            let (token, rest) = if let Some(next) = self.find('.') {
                (&self[..next], &self[next+1..])
            } else if let Some(next) = self.find('[') {
                (&self[..next], &self[next..])
            } else {
                (self, "")
            };
            if token == "" {
                (PathToken::Empty, rest)
            } else {
                (PathToken::Ident(token), rest)
            }


        }
    }
}

#[derive(Debug)]
pub enum PathToken<'s> {
    Ident(&'s str),
    Index(usize),
    Empty,
    Invalid,
}
pub struct DynamicProp<'w, 'o, R> {
    read: Rc<dyn Fn() -> Box<dyn Reflect> + 'o>,
    write: Rc<dyn Fn(Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> + 'o>,
    marker: PhantomData<(&'w (), R)>
}

impl<'w, 'o, R> DynamicProp<'w, 'o, R> {
    pub fn get(&self) -> Box<dyn Reflect> {
        (self.read)()
    }
}

impl<'w, 'o> DynamicProp<'w, 'o, Mutable> {
    pub fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        (self.write)(value)
    }

    pub fn update(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        let eq = self.get().reflect_partial_eq(value.as_ref());
        if eq.is_none() || !eq.unwrap() {
            self.set(value)
        } else {
            Ok(())
        }
    }
}

impl<'w, 'o, T: Reflect + FromReflect, R> From<Prop<'w, 'o, T, R>> for DynamicProp<'w, 'o, R> {
    fn from(value: Prop<'w, 'o, T, R>) -> Self {
        use usage::*;
        let read = value.reader();
        let write = value.writer();
        let read = Rc::new(move || read.untyped());
        let write = Rc::new(move |reflected: Box<dyn Reflect>| {
            write.untyped(reflected)
        });
        Self { read, write, marker: PhantomData }
    }
}

pub trait DynamicProps<'w, 'o, R> {
    fn lookup<S: AsRef<str>>(self, path: S) -> Option<DynamicProp<'w, 'o, R>>;
}

impl<'w, 'o, T: PartialEq, R> PartialEq<T> for Prop<'w, 'o, T, R> {
    fn eq(&self, other: &T) -> bool {
        let mut eq = false;
        use usage::ImutableProp;
        self.with(|v| eq = v == other);
        eq
    }
}

pub trait Props<'w, 'o, R> where Self: Sized {
    type Impl: DynamicProps<'w, 'o, R>;
    fn make_prop(prop: Prop<'w, 'o, Self, R>) -> Self::Impl;
}


pub trait ExtractProps<'w, 'o, R> {
    type Impl;
    fn props(self) -> Self::Impl;
}

impl<'w, 'o, T: Component + Props<'w, 'o, Imutable>> ExtractProps<'w, 'o, Imutable> for &'o T {
    type Impl = T::Impl;
    fn props(self) -> Self::Impl {
        T::make_prop(Prop::from_ref(self))
    }
}

impl<'w, 'o, T: Component + Props<'w, 'o, Mutable>> ExtractProps<'w, 'o, Mutable> for &'o mut Mut<'w, T> {
    type Impl = T::Impl;
    fn props(self) -> Self::Impl {
        T::make_prop(Prop::from_mut(self))
    }
}