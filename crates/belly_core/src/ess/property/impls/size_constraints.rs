use super::parse;
use crate::style_property;
use bevy::prelude::*;

// impl_style_single_value!("width", WidthProperty, Val, val, size.width);
style_property! {
    #[doc = " Specify element preferred width by providing value to `Style.size.width`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " width: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    WidthProperty("width") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.size.width != value {
                style.size.width = *value;
            }
        };
    }
}
// impl_style_single_value!("height", HeightProperty, Val, val, size.height);
style_property! {
    #[doc = " Specify element preferred height by providing value to `Style.size.height`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " height: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    HeightProperty("height") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.size.height != value {
                style.size.height = *value;
            }
        };
    }
}

// impl_style_single_value!("min-width", MinWidthProperty, Val, val, min_size.width);
style_property! {
    #[doc = " Specify element minimum width by providing value to `Style.min_size.width`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " min-width: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    MinWidthProperty("min-width") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.min_size.width != value {
                style.min_size.width = *value;
            }
        };
    }
}
// impl_style_single_value!("min-height", MinHeightProperty, Val, val, min_size.height);
style_property! {
    #[doc = " Specify element minimum height by providing value to `Style.min_size.height`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " min-height: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    MinHeightProperty("min-height") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.min_size.height != value {
                style.min_size.height = *value;
            }
        };
    }
}

// impl_style_single_value!("max-width", MaxWidthProperty, Val, val, max_size.width);
style_property! {
    #[doc = " Specify element maximum width by providing value to `Style.max_size.width`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " max-width: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    MaxWidthProperty("max-width") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.max_size.width != value {
                style.max_size.width = *value;
            }
        };
    }
}
// impl_style_single_value!("max-height", MaxHeightProperty, Val, val, max_size.height);
style_property! {
    #[doc = " Specify element maximum height by providing value to `Style.max_size.height`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " max-height: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    MaxHeightProperty("max-height") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.max_size.height != value {
                style.max_size.height = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element preferred aspect ratio by providing value to"]
    #[doc = " `Style.aspect_ratio`:"]
    #[doc = " ```css"]
    #[doc = " aspect-ratio: 2.0;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " The `aspect-ratio` property sets a preferred aspect ratio for"]
    #[doc = " the box, which will be used in the calculation of auto sizes"]
    #[doc = " and some other layout functions."]
    AspectRatioProperty("aspect-ratio") {
        Default = "none";
        Item = Option<f32>;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::OptionalNumParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.aspect_ratio != value {
                style.aspect_ratio = *value;
            }
        };
    }
}
