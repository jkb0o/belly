use bevy::reflect::TypePath;

use super::usage::*;

pub struct OptionProps<'w, 'o, R, I>(Prop<'w, 'o, Option<I>, R>);
impl<'w, 'o, R: 'static, I: Props<'w, 'o, R> + Reflect + FromReflect + TypePath + Default + 'o> Props<'w, 'o, R> for Option<I> {
    type Impl = OptionProps<'w, 'o, R, I>;
    fn make_prop(prop: Prop<'w, 'o, Self, R>) -> Self::Impl {
        OptionProps(prop)
    }
}
impl<'w, 'o, R: 'static, I: Props<'w, 'o, R> + Default + 'o>OptionProps<'w, 'o, R, I> {
    pub fn value(self) -> <I as Props<'w, 'o, R>>::Impl {
        let write = match self.0.writer() {
            Write::Mut(mutator) => Write::mutator(move || {
                let value = mutator();
                if value.is_none() {
                    *value = Some(I::default())
                }
                value.as_mut().unwrap()
            }),
            Write::Set(setter) => Write::setter(move |value| setter(Some(value))),
        };
        let getter = self.0.reader();
        let read = match getter {
            Read::Ref(getter) => Read::reference(move || getter().as_ref().unwrap()),
            Read::Val(getter) => Read::Val(Rc::new(move || getter().unwrap_or_default()))
        };
        I::make_prop(Prop::new(read, write))
    }
}

impl<'w, 'o, R: 'static, I: Props<'w, 'o, R> + FromReflect + Reflect + TypePath + Default + 'o> DynamicProps<'w, 'o, R> for OptionProps<'w, 'o, R, I>
{
    fn lookup<S: AsRef<str>>(self, path: S) -> Option<DynamicProp<'w, 'o, R>> {
        let (token, rest) = path.as_ref().parse_token();
        match token {
            PathToken::Empty => Some(self.0.into()),
            PathToken::Ident("value") => self.value().lookup(rest),
            _ => None
        }
    }
}

pub struct VecProps<'w, 'o, R, I>(Prop<'w, 'o, Vec<I>, R>);
impl<'w, 'o, R: 'static, I: Props<'w, 'o, R> + Reflect + FromReflect + TypePath + 'o> Props<'w, 'o, R> for Vec<I> {
    type Impl = VecProps<'w, 'o, R, I>;
    fn make_prop(prop: Prop<'w, 'o, Self, R>) -> Self::Impl {
        VecProps(prop)
    }
}

impl<'w, 'o, R: 'static, I: Props<'w, 'o, R> + 'o>VecProps<'w, 'o, R, I> {
    pub fn at(self, idx: usize) -> <I as Props<'w, 'o, R>>::Impl {
        let write = match self.0.writer() {
            Write::Mut(mutator) => Write::mutator(move || mutator().get_mut(idx).unwrap()),
            Write::Set(_setter) => Write::mutator(move || panic!("Setter-based vector props not supported."))
        };
        let read = match self.0.reader() {
            Read::Ref(getref) => Read::reference(move || getref().get(idx).unwrap()),
            Read::Val(_getval) => Read::reference(move || panic!("Setter-based vector props not supported."))
        };
        <I as Props<'w, 'o, R>>::make_prop(Prop::new(read, write))
    }
}

impl<'w, 'o, R: 'static, I: Props<'w, 'o, R> + FromReflect + Reflect + TypePath + 'o> DynamicProps<'w, 'o, R> for VecProps<'w, 'o, R, I>
{
    fn lookup<S: AsRef<str>>(self, path: S) -> Option<DynamicProp<'w, 'o, R>> {
        let (token, rest) = path.as_ref().parse_token();
        match token {
            PathToken::Empty => Some(self.0.into()),
            PathToken::Index(idx) => self.at(idx).lookup(rest),
            _ => None
        }
    }
}

macro_rules! impl_primitive {
    ($t:ty) => {
        impl<'w, 'o, R: 'static> Props<'w, 'o, R> for $t {
            type Impl = Prop<'w, 'o, $t, R>;
            fn make_prop(prop: Prop<'w, 'o, Self, R>) -> Self::Impl {
                prop
            }
        }
        impl<'w, 'o, R: 'static> DynamicProps<'w, 'o, R> for Prop<'w, 'o, $t, R> {
            fn lookup<S: AsRef<str>>(self, path: S) -> Option<DynamicProp<'w, 'o, R>> {
                if path.as_ref().is_empty() {
                    Some(self.into())
                } else {
                    None
                }
            }
        }
    };
}

impl_primitive!(bool);
impl_primitive!(String);
impl_primitive!(f32);
impl_primitive!(f64);
impl_primitive!(usize);
impl_primitive!(isize);
impl_primitive!(u128);
impl_primitive!(i128);
impl_primitive!(u64);
impl_primitive!(i64);
impl_primitive!(u32);
impl_primitive!(i32);
impl_primitive!(u16);
impl_primitive!(i16);
impl_primitive!(u8);
impl_primitive!(i8);


macro_rules! impl_type {
    (@lookup$( $($docs:literal)* $p:ident : $t:ty => $def:tt;)+) => {
        fn lookup<S: AsRef<str>>(self, path: S) -> Option<DynamicProp<'w, 's, R>> {
            let (token, rest) = path.as_ref().parse_token();
            match token {
                PathToken::Empty => Some(self.0.into()),
                $(
                    
                    PathToken::Ident(stringify!($p)) => { self.$p().lookup(rest) },
                )+
                _ => None
            }
        }
    };
    (@method) => { };
    (@method $($doc:literal)* $p:ident : $t:ty => [$field:tt]; $($rest:tt)*) => {
        $(#[doc = $doc])*
        pub fn $p(self) -> <$t as Props<'w, 'o, R>>::Impl {
            let write = match self.0.writer() {
                Write::Mut(mutator) => {
                    Write::mutator(move || &mut mutator().$field)
                },
                Write::Set(setter) => {
                    let getter = self.0.reader();
                    Write::setter(move |$p| {
                        let mut this = match &getter {
                            Read::Ref(get_ref) => get_ref().clone(),
                            Read::Val(get_value) => get_value(),
                        };
                        this.$field = $p;
                        setter(this)
                    })
                }
            };
            let getter = self.0.reader();
            let read = match getter {
                Read::Ref(getter) => Read::Ref(Rc::new(move || &getter().$field)),
                Read::Val(getter) => Read::Val(Rc::new(move || getter().$field))
            };
            <$t>::make_prop(Prop::new(read, write))
        }
        impl_type! { @method $($rest)* }
    };
    // (@method $($doc:literal)* $p:ident : $t:ident => [$vec:ident[$vectype:ty]]; $($rest:tt)*) => {
    // }
    (@method $($doc:literal)* $p:ident : $t:ident => [$getter:ident, $setter:ident]; $($rest:tt)*) => {
        $(#[doc = $doc])*
        pub fn $p(self) -> <$t as Props<'w, 'o, R>>::Impl {
            let write = match self.0.writer() {
                Write::Mut(mutator) => {
                    Write::setter(move |$p| { mutator().$setter($p); })
                },
                Write::Set(setter) => {
                    let getter = self.0.reader();
                    Write::setter(move |$p| {
                        let mut this = match &getter {
                            Read::Ref(get_ref) => get_ref().clone(),
                            Read::Val(get_value) => get_value(),
                        };
                        this.$setter($p);
                        setter(this)
                    })
                }
            };
            let getter = self.0.read.clone();
            let read = Read::value(move || match &getter {
                Read::Ref(get_ref) => get_ref().$getter(),
                Read::Val(get_value) => get_value().$getter(),
            });
            $t::make_prop(Prop::new(read, write))
        }
        impl_type! { @method $($rest)* }
    };
    // (@method $($doc:literal)* $p:ident : $t:ident => [$field:ident[]]; $($rest:tt)*) => {
    // };
    ($typ:ty; $props:ident; $($def:tt)+) => {
        #[derive(Deref, DerefMut)]
        pub struct $props<'w, 'o, R>(Prop<'w, 'o, $typ, R>);

        impl<'w, 'o, R: 'static> Props<'w, 'o, R> for $typ {
            type Impl = $props<'w, 'o, R>;
            fn make_prop(prop: Prop<'w, 'o, Self, R>) -> Self::Impl {
                $props(prop)
            }
        }

        impl<'w, 'o, R: 'static> $props<'w, 'o, R> {
            impl_type! { @method $($def)+ }
        }
        impl<'w, 's, R: 'static> DynamicProps<'w, 's, R> for $props<'w, 's, R> {
            impl_type! { @lookup $($def)+ }
        }
        
    };
}


impl_type! { Color; ColorProps;
    " Red channel. [0.0, 1.0]"
    r:f32 => [r, set_r];
    " Green channel. [0.0, 1.0]"
    g:f32 => [g, set_g];
    " Blue channel. [0.0, 1.0]"
    b:f32 => [b, set_b];
    " Alpha channel. [0.0, 1.0]"
    a:f32 => [a, set_a];
}

impl_type! { Vec3; Vec3Props;
    x:f32 => [x];
    y:f32 => [y];
    z:f32 => [z];
}

impl_type! { Quat; QuatProps;
    x:f32 => [x];
    y:f32 => [y];
    z:f32 => [z];
    w:f32 => [w];
}

impl_type! { Transform; TransformProps;
    " Position of the entity. In 2d, the last value of the"
    " `Vec3` is used for z-ordering."
    translation:Vec3 => [translation];
    " Rotation of the entity."
    rotation:Quat => [rotation];
    " Scale of the entity."
    scale:Vec3 => [scale];
}

impl_type!{ BackgroundColor; BackgroundColorProps;
    " The background color of the node"
    " "
    " This serves as the \"fill\" color."
    color:Color => [0];
}

trait ValExtension {
    fn is_auto(&self) -> bool;
    fn set_auto(&mut self, value: bool);
    fn px(&self) -> f32;
    fn set_px(&mut self, value: f32);
    fn percent(&self) -> f32;
    fn set_percent(&mut self, value: f32);
    fn vw(&self) -> f32;
    fn set_vw(&mut self, value: f32);
    fn vh(&self) -> f32;
    fn set_vh(&mut self, value: f32);
    fn vmin(&self) -> f32;
    fn set_vmin(&mut self, value: f32);
    fn vmax(&self) -> f32;
    fn set_vmax(&mut self, value: f32);
}
impl ValExtension for Val {
    fn is_auto(&self) -> bool {
        match self {
            Val::Auto => true,
            _ => false
        }
    }
    fn set_auto(&mut self, value: bool) {
        if value {
            *self = Val::Auto
        } else {
            *self = Val::Px(0.)
        }
    }
    fn px(&self) -> f32 {
        match self {
            Val::Px(px) => *px,
            _ => 0.
        }
    }
    fn set_px(&mut self, value: f32) {
        *self = Val::Px(value)
    }
    fn percent(&self) -> f32 {
        match self {
            Val::Percent(percent) => *percent,
            _ => 0.
        }
    }
    fn set_percent(&mut self, value: f32) {
        *self = Val::Percent(value)
    }
    fn vw(&self) -> f32 {
        match self {
            Val::Vw(vw) => *vw,
            _ => 0.
        }
    }
    fn set_vw(&mut self, value: f32) {
        *self = Val::Vw(value)
    }
    fn vh(&self) -> f32 {
        match self {
            Val::Vh(vh) => *vh,
            _ => 0.
        }
    }
    fn set_vh(&mut self, value: f32) {
        *self = Val::Vh(value)
    }
    fn vmin(&self) -> f32 {
        match self {
            Val::VMin(vmin) => *vmin,
            _ => 0.
        }
    }
    fn set_vmin(&mut self, value: f32) {
        *self = Val::VMin(value)
    }
    fn vmax(&self) -> f32 {
        match self {
            Val::VMax(vmax) => *vmax,
            _ => 0.
        }
    }
    fn set_vmax(&mut self, value: f32) {
        *self = Val::VMax(value)
    }
}
impl_type!(Val; ValProps;
    " Automatically determine the value based on the context and other [`Style`] properties."
    " Setting this to `false` converts value to Val::Px(0.)."
    auto:bool => [is_auto, set_auto];
    " Set this value in logical pixels."
    px:f32 => [px, set_px];
    " Set the value as a percentage of its parent node's length along a specific axis."
    " "
    " If the UI node has no parent, the percentage is calculated based on the window's length"
    " along the corresponding axis."
    " "
    " The chosen axis depends on the `Style` field set:"
    " * For `flex_basis`, the percentage is relative to the main-axis length determined by the `flex_direction`."
    " * For `gap`, `min_size`, `size`, and `max_size`:"
    "   - `width` is relative to the parent's width."
    "   - `height` is relative to the parent's height."
    " * For `margin`, `padding`, and `border` values: the percentage is relative to the parent node's width."
    " * For positions, `left` and `right` are relative to the parent's width, while `bottom` and `top` are relative to the parent's height."
    percent:f32 => [percent, set_percent];
    " Set this value in percent of the viewport width"
    vw:f32 => [vw, set_vw];
    " Set this value in percent of the viewport height"
    vh:f32 => [vh, set_vh];
    " Set this value in percent of the viewport's smaller dimension."
    vmin:f32 => [vmin, set_vmin];
    " Set this value in percent of the viewport's larger dimension."
    vmax:f32 => [vmax, set_vmax];
);


impl_primitive!(Display);
impl_primitive!(PositionType);
impl_primitive!(Overflow);
impl_primitive!(Direction);


impl_primitive!(GridTrack);


impl_type! { Style; StyleProps;
    " Which layout algorithm to use when laying out this node's contents:"
    "   - [`Display::Flex`]: Use the Flexbox layout algorithm"
    "   - [`Display::Grid`]: Use the CSS Grid layout algorithm"
    "   - [`Display::None`]: Hide this node and perform layout as if it does not exist."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/display>"
    display:Display => [ display ];

    " Whether a node should be laid out in-flow with, or independently of it's siblings:"
    "  - [`PositionType::Relative`]: Layout this node in-flow with other nodes using the usual (flexbox/grid) layout algorithm."
    "  - [`PositionType::Absolute`]: Layout this node on top and independently of other nodes."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/position>"
    position_type:PositionType => [ position_type ];

    " Whether overflowing content should be displayed or clipped."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/overflow>"
    overflow: Overflow => [ overflow ];

    " Defines the text direction. For example English is written LTR (left-to-right) while Arabic is written RTL (right-to-left)."
    " "
    " Note: the corresponding CSS property also affects box layout order, but this isn't yet implemented in bevy."
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/direction>"
    direction: Direction => [ direction ];
    
    " The horizontal position of the left edge of the node."
    " - For relatively positioned nodes, this is relative to the node's position as computed during regular layout."
    " - For absolutely positioned nodes, this is relative to the *parent* node's bounding box."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/left>"
    left: Val => [ left ];

    " The horizontal position of the right edge of the node."
    "  - For relatively positioned nodes, this is relative to the node's position as computed during regular layout."
    "  - For absolutely positioned nodes, this is relative to the *parent* node's bounding box."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/right>"
    right: Val => [ right ];

    " The vertical position of the top edge of the node."
    " - For relatively positioned nodes, this is relative to the node's position as computed during regular layout."
    " - For absolutely positioned nodes, this is relative to the *parent* node's bounding box."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/top>"
    top: Val => [ top ];

    " The vertical position of the bottom edge of the node."
    " - For relatively positioned nodes, this is relative to the node's position as computed during regular layout."
    " - For absolutely positioned nodes, this is relative to the *parent* node's bounding box."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/bottom>"
    bottom: Val => [ bottom ];

    " The ideal width of the node. `width` is used when it is within the bounds defined by `min_width` and `max_width`."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/width>"
    width:Val => [ width ];

    " The ideal height of the node. `height` is used when it is within the bounds defined by `min_height` and `max_height`."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/height>"
    height: Val => [ height ];

    " The minimum width of the node. `min_width` is used if it is greater than either `width` and/or `max_width`."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/min-width>"
    min_width: Val => [ min_width ];

    " The minimum height of the node. `min_height` is used if it is greater than either `height` and/or `max_height`."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/min-height>"
    min_height: Val => [ min_height ];

    " The maximum width of the node. `max_width` is used if it is within the bounds defined by `min_width` and `width`."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/max-width>"
    max_width: Val => [ max_width ];

    " The maximum height of the node. `max_height` is used if it is within the bounds defined by `min_height` and `height`."
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/max-height>"
    max_height: Val => [ max_height ];

    " The aspect ratio of the node (defined as `width / height`)"
    " "
    " <https://developer.mozilla.org/en-US/docs/Web/CSS/aspect-ratio>"
    aspect_ratio: Option<f32> => [ aspect_ratio ];






    grid_auto_rows: Vec<GridTrack> => [ grid_auto_rows ];
}