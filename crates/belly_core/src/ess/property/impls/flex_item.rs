use super::parse;
use crate::style_property;
use bevy::prelude::*;

style_property! {
    #[doc = " TODO: write AlignSelf description"]
    AlignSelfProperty("align-self") {
        Default = "auto";
        Item = AlignSelf;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::IdentifierParser<AlignSelf>;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.align_self != value {
                style.align_self = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element flex grow by providing value to `Style.flex_grow`:"]
    #[doc = " ```css"]
    #[doc = " flex-grow: 2.0;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " The `flex-grow` defines the ability for a flex item to grow if necessary."]
    #[doc = " It accepts a unitless value that serves as a proportion. It dictates what"]
    #[doc = " amount of the available space inside the flex container the item should"]
    #[doc = " take up."]

    FlexGrowProperty("flex-grow") {
        Default = "0.0";
        Item = f32;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::NumParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.flex_grow != value {
                style.flex_grow = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element flex shrink by providing value to `Style.flex_shrink`:"]
    #[doc = " ```css"]
    #[doc = " flex-shrink: 3.0;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " The flex-shrink property specifies how the item will shrink relative to"]
    #[doc = " the rest of the flexible items inside the same container."]
    FlexShrinkProperty("flex-shrink") {
        Default = "1.0";
        Item = f32;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::NumParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.flex_shrink != value {
                style.flex_shrink = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element flex basis by providing value to `Style.flex_basis`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " flex-basis: 5px;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " The `flex-basis` specifies the initial size of the flex item, before"]
    #[doc = " any available space is distributed according to the flex factors."]
    #[doc = " <!-- (TODO: link val) -->"]
    FlexBasisProperty("flex-basis") {
        Default = "auto";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.flex_basis != value {
                style.flex_basis = *value;
            }
        };
    }
}
