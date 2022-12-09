use std::{fmt::Debug, mem};

use crate::property::*;
use crate::tags;
use crate::variant;
use crate::variant::ApplyCommands;
use crate::ElementsError;
use crate::Variant;
use bevy::prelude::error;
use bevy::{
    ecs::system::EntityCommands,
    utils::{hashbrown::hash_map::Drain, HashMap, HashSet},
};
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
                value: Variant::Empty,
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

    pub fn take<T: 'static>(&mut self) -> Option<T> {
        mem::take(&mut self.value).take()
    }

    pub fn take_varint(&mut self) -> Variant {
        mem::take(&mut self.value)
    }
}

// fn test_system
#[derive(Default, Debug)]
pub struct Params(HashMap<Tag, Param>);

impl Params {
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
            if let Some(existed) = self.get_mut::<String>(attr.name) {
                if let Some(classes) = attr.take::<String>() {
                    existed.push_str(" ");
                    existed.push_str(classes.as_str());
                    return;
                }
            }
        }
        match attr.target {
            ParamTarget::Param => self.0.insert(attr.name, attr),
            ParamTarget::Class => match self.0.get_mut(&tags::class()) {
                Some(class) => {
                    let classes = class
                        .value
                        .get_mut::<String>()
                        .expect("Class param should be of type String.");
                    classes.push_str(" ");
                    classes.push_str(attr.name.into());
                    None
                }
                None => {
                    attr = Param::new(tags::class().into(), Variant::String(attr.name.into()));
                    self.0.insert(tags::class(), attr)
                }
            },
            ParamTarget::Style => match self.0.get_mut(&tags::styles()) {
                Some(styles) => {
                    let styles = styles
                        .value
                        .get_mut::<Params>()
                        .expect("Styles param should be of type Params.");
                    attr.target = ParamTarget::Param;
                    styles.add(attr);
                    None
                }
                None => {
                    let mut styles = Params::default();
                    attr.target = ParamTarget::Param;
                    styles.add(attr);
                    let attr = Param::new(tags::styles().into(), Variant::Params(styles));
                    self.0.insert(tags::styles(), attr)
                }
            },
        };
    }

    pub fn drain(&mut self) -> Drain<Tag, Param> {
        self.0.drain()
    }

    pub fn merge(&mut self, mut other: Self) {
        if let Some(other_classes) = other.0.remove(&tags::class()) {
            if let Some(self_classes) = self.0.get_mut(&tags::class()) {
                let self_class_string = self_classes
                    .value
                    .get_mut::<String>()
                    .expect("Class param should be of type String.");
                let other_class_string = other_classes
                    .value
                    .get::<String>()
                    .expect("Class param should be of type String.");
                self_class_string.push_str(" ");
                self_class_string.push_str(other_class_string.as_str());
            } else {
                self.0.insert(tags::class(), other_classes);
            }
        }
        if let Some(mut other_styles) = other.0.remove(&tags::styles()) {
            if let Some(self_styles) = self.0.get_mut(&tags::styles()) {
                let self_styles_value = self_styles
                    .value
                    .get_mut::<Params>()
                    .expect("styles param should be of type Params");
                let other_styles_value = other_styles
                    .value
                    .get_mut::<Params>()
                    .expect("styles param should be of type Params");
                for (_, attr) in other_styles_value.0.drain() {
                    self_styles_value.add(attr);
                }
            } else {
                self.0.insert(tags::styles(), other_styles);
            }
        }
        for (name, attr) in other.0.drain() {
            if let Some(self_attr) = self.0.get_mut(&name) {
                self_attr.value.merge(attr.value);
            } else {
                self.add(attr);
            }
        }
    }

    pub fn commands(&mut self, name: Tag) -> Option<ApplyCommands> {
        self.drop::<ApplyCommands>(name)
    }

    pub fn styles(&mut self) -> Params {
        self.drop::<Params>(tags::styles()).unwrap_or_default()
    }
    pub fn classes(&mut self) -> HashSet<Tag> {
        self.drop::<String>(tags::class())
            .unwrap_or("".to_string())
            .split(" ")
            .filter(|s| !s.is_empty())
            .map(|s| s.as_tag())
            .collect()
    }
    pub fn id(&mut self) -> Option<Tag> {
        self.drop(tags::id())
    }
    pub fn get<T: 'static>(&self, key: Tag) -> Option<&T> {
        self.0.get(&key).and_then(|v| v.value.get::<T>())
    }
    pub fn get_variant(&self, key: Tag) -> Option<&Variant> {
        self.0.get(&key).map(|v| &v.value)
    }
    pub fn get_mut<T: 'static>(&mut self, key: Tag) -> Option<&mut T> {
        self.0.get_mut(&key).and_then(|v| v.value.get_mut::<T>())
    }
    pub fn drop<T: 'static>(&mut self, key: Tag) -> Option<T> {
        self.0.remove(&key).and_then(|mut a| a.take())
    }
    pub fn drop_variant(&mut self, key: Tag) -> Option<Variant> {
        self.0.remove(&key).map(|mut a| a.take_varint())
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

    pub fn contains(&self, tag: Tag) -> bool {
        self.0.contains_key(&tag)
    }

    pub fn transform<I: IntoIterator<Item = (Tag, Variant)>, F: Fn(Tag, Variant) -> I>(
        mut self,
        transform: F,
    ) -> Params {
        let mut this = Params::default();
        for (tag, mut param) in self.0.drain() {
            for (tag, variant) in transform(tag, param.take_varint()) {
                this.add(Param {
                    name: tag,
                    value: variant,
                    target: param.target,
                })
            }
        }
        this
    }
}

#[macro_export]
macro_rules! bindattr {
    ($ctx:ident, $key:ident:$typ:ident => $($target:tt)*) => {
        {
            let __elem = $ctx.entity();
            let __key = stringify!($key).as_tag();
            let __attr = $ctx.param(__key);
            let mut __value = Default::default();
            match __attr {
                Some($crate::Variant::BindFrom(__b)) => $ctx.commands().add(__b.to($crate::bind!(=> __elem, $($target)*))),
                Some($crate::Variant::BindTo(__b)) => $ctx.commands().add(__b.from($crate::bind!(<= __elem, $($target)*))),
                Some(__attr) => match $typ::try_from(__attr) {
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
        attrs.add(Param::new("c:some-other-class", Variant::Empty));
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
        attrs.add(Param::new("c:some-other-class", Variant::Empty));
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
        let styles = attrs.drop::<Params>("styles".as_tag());
        assert!(styles.is_some());
        let styles = styles.unwrap();
        assert_eq!(
            styles.get::<String>("color".as_tag()),
            Some(&"black".to_string())
        );
    }
}
