use crate::eml::ApplyCommands;
use crate::eml::Variant;
use crate::ess::*;
use crate::tags;
use bevy::log::error;
use bevy::prelude::Deref;
use bevy::prelude::DerefMut;
use bevy::{
    ecs::system::EntityCommands,
    utils::{HashMap, HashSet},
};
use std::fmt::Display;
use std::{fmt::Debug, mem};
use tagstr::*;

#[derive(Debug, Clone, Copy)]
pub enum ParamTarget {
    Param,
    Style,
    Class,
}

#[derive(Debug)]
pub struct Param {
    name: Tag,
    value: Variant,
    target: ParamTarget,
}

impl Param {
    pub fn from_commands(name: &str, commands: ApplyCommands) -> Param {
        let value = Variant::Commands(commands);
        Param {
            name: name.as_tag(),
            value,
            target: ParamTarget::Param,
        }
    }
    pub fn new(name: &str, value: Variant) -> Param {
        if name.starts_with("c:") {
            Param {
                name: name.strip_prefix("c:").unwrap().as_tag(),
                value: Variant::Undefined,
                target: ParamTarget::Class,
            }
        } else if name.starts_with("s:") {
            Param {
                name: name.strip_prefix("s:").unwrap().as_tag(),
                value,
                target: ParamTarget::Style,
            }
        } else {
            Param {
                value,
                name: name.as_tag(),
                target: ParamTarget::Param,
            }
        }
    }

    pub fn style(name: Tag, value: Variant) -> Param {
        Param {
            name,
            value,
            target: ParamTarget::Style,
        }
    }

    pub fn take<T: 'static>(&mut self) -> Option<T> {
        mem::take(&mut self.value).take()
    }

    pub fn take_varint(&mut self) -> Variant {
        mem::take(&mut self.value)
    }
}

#[derive(Default, Deref, DerefMut, Debug)]
pub struct StyleParams(HashMap<Tag, Variant>);

impl StyleParams {
    pub fn transform<I: IntoIterator<Item = (Tag, PropertyValue)>, F: Fn(Tag, Variant) -> I>(
        self,
        transform: F,
    ) -> HashMap<Tag, PropertyValue> {
        let mut styles = HashMap::default();
        for (tag, param) in self.0 {
            for (tag, property) in transform(tag, param) {
                styles.insert(tag, property);
            }
        }
        styles
    }
}

// fn test_system
#[derive(Default, Debug)]
pub struct Params {
    pub(crate) defined_classes: HashSet<Tag>,
    pub(crate) defined_styles: StyleParams,
    pub(crate) rest: HashMap<Tag, Param>,
}

impl Params {
    pub fn insert(&mut self, name: &str, value: Variant) {
        self.add(Param::new(name, value))
    }
    pub fn add(&mut self, mut attr: Param) {
        if attr.name == tags::params() {
            if let Some(mut attrs) = attr.take::<Params>() {
                let this = mem::take(self);
                attrs.merge(this);
                *self = attrs;
                return;
            } else {
                panic!("params should be of type Params.")
            }
        }
        if attr.name == tags::class() {
            if let Variant::String(classes) = attr.value {
                for class in classes.split_whitespace() {
                    self.defined_classes.insert(class.as_tag());
                }
                return;
            }
        }
        match attr.target {
            ParamTarget::Param => {
                self.rest.insert(attr.name, attr);
            }
            ParamTarget::Style => {
                self.defined_styles.insert(attr.name, attr.value);
            }
            ParamTarget::Class => {
                self.defined_classes.insert(attr.name);
            }
        }
    }

    // pub fn drain(&mut self) -> Drain<Tag, Param> {
    //     self.0.drain()
    // }

    pub fn merge(&mut self, mut other: Self) {
        self.defined_classes.extend(other.defined_classes);
        self.defined_styles.extend(other.defined_styles.0);
        for (name, value) in other.rest.drain() {
            if let Some(param) = self.rest.get_mut(&name) {
                param.value.merge(value.value);
            } else {
                self.rest.insert(name, value);
            }
        }
    }

    pub fn commands(&mut self, name: Tag) -> Option<ApplyCommands> {
        self.drop::<ApplyCommands>(name)
    }

    pub fn classes(&mut self) -> HashSet<Tag> {
        mem::take(&mut self.defined_classes)
    }

    pub fn styles(&mut self) -> StyleParams {
        mem::take(&mut self.defined_styles)
    }

    pub fn id(&mut self) -> Option<Tag> {
        self.drop::<String>(tags::id()).map(|s| s.into())
    }
    pub fn get<T: 'static>(&self, key: Tag) -> Option<&T> {
        self.rest.get(&key).and_then(|v| v.value.get::<T>())
    }
    pub fn get_variant(&self, key: Tag) -> Option<&Variant> {
        self.rest.get(&key).map(|v| &v.value)
    }
    pub fn get_mut<T: 'static>(&mut self, key: Tag) -> Option<&mut T> {
        self.rest.get_mut(&key).and_then(|v| v.value.get_mut::<T>())
    }
    pub fn drop<T: 'static>(&mut self, key: Tag) -> Option<T> {
        self.rest.remove(&key).and_then(|mut a| a.take())
    }
    pub fn drop_variant(&mut self, key: Tag) -> Option<Variant> {
        self.rest.remove(&key).map(|mut a| a.take_varint())
    }
    pub fn drop_or_default<T: 'static>(&mut self, key: Tag, default: T) -> T {
        if let Some(value) = self.drop(key) {
            value
        } else {
            default
        }
    }
    pub fn apply_commands(&mut self, for_param: Tag, commands: &mut EntityCommands) {
        if let Some(param_commands) = self.commands(for_param) {
            param_commands(commands)
        }
    }
    pub fn try_get<T: TryFrom<Variant, Error = impl Display>>(&mut self, param: &str) -> Option<T> {
        if let Some(value) = self.drop_variant(param.as_tag()) {
            match T::try_from(value) {
                Ok(value) => Some(value),
                Err(e) => {
                    error!("Invalid value for '{param}' param: {e}");
                    None
                }
            }
        } else {
            None
        }
    }

    // pub fn contains(&self, tag: Tag) -> bool {
    //     self.rest.contains_key(&tag)
    // }

    // pub fn transform<I: IntoIterator<Item = (Tag, PropertyValue)>, F: Fn(Tag, Variant) -> I>(
    //     mut self,
    //     transform: F,
    // ) -> HashMap<Tag, PropertyValue> {
    //     let mut this = HashMap::default();
    //     for (tag, mut param) in self.rest.drain() {
    //         for (tag, variant) in transform(tag, param.take_varint()) {
    //             this.insert(tag, variant);
    //         }
    //     }
    //     this
    // }
}

#[macro_export]
macro_rules! bindattr {
    ($ctx:ident, $key:ident:$typ:ty => $($target:tt)*) => {
        {
            let __elem = $ctx.entity();
            let __key = stringify!($key).as_tag();
            let __attr = $ctx.param(__key);
            let mut __value = Default::default();
            match __attr {
                Some($crate::Variant::BindFrom(__b)) => $ctx.commands().add(__b.to($crate::bind!(=> __elem, $($target)*))),
                Some($crate::Variant::BindTo(__b)) => $ctx.commands().add(__b.from($crate::bind!(<= __elem, $($target)*))),
                Some(__attr) => match <$typ>::try_from(__attr) {
                    Ok(__v) => __value = Some(__v),
                    Err(__err) => error!("Invalid value for '{}' param: {}", __key, __err)
                },
                _ => ()
            };
            __value
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_classes() {
        let mut attrs = Params::default();
        attrs.add(Param::new("class", "some-class".into()));
        assert_eq!(
            attrs.classes(),
            ["some-class".as_tag()].iter().cloned().collect()
        );
    }

    #[test]
    fn test_c_not_overrides_class() {
        let mut attrs = Params::default();
        attrs.add(Param::new("class", "class1 class2".into()));
        attrs.add(Param::new("c:some-other-class", Variant::Undefined));
        assert_eq!(
            attrs.classes(),
            ["class1", "class2", "some-other-class"]
                .iter()
                .map(|s| s.as_tag())
                .collect()
        );
    }

    #[test]
    fn test_class_not_overrides_c() {
        let mut attrs = Params::default();
        attrs.add(Param::new("c:some-other-class", Variant::Undefined));
        attrs.add(Param::new("class", "class1 class2".into()));
        assert_eq!(
            attrs.classes(),
            ["class1", "class2", "some-other-class"]
                .iter()
                .map(|s| s.as_tag())
                .collect()
        );
    }

    #[test]
    fn test_basic_styles() {
        let mut attrs = Params::default();
        attrs.add(Param::new("s:color", "black".into()));
        let styles = attrs.styles();
        assert!(!styles.is_empty());
        assert_eq!(
            styles.get(&"color".as_tag()).unwrap().get::<String>(),
            Some(&"black".to_string())
        );
    }
}
