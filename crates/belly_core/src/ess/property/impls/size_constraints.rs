use super::parse;
use crate::style_property;
use bevy::prelude::*;

// impl_style_single_value!("width", WidthProperty, Val, val, size.width);
style_property! {
    #[doc = " Specify element preferred width by providing value to `Style.width`:"]
    #[doc = " ```css"]
    #[doc = " width: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Size Constraints -->"]
    WidthProperty("width") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.width != value {
                style.width = *value;
            }
        };
    }
}
// impl_style_single_value!("height", HeightProperty, Val, val, size.height);
style_property! {
    #[doc = " Specify element preferred height by providing value to `Style.height`:"]
    #[doc = " ```css"]
    #[doc = " height: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Size Constraints -->"]
    HeightProperty("height") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.height != value {
                style.height = *value;
            }
        };
    }
}

// impl_style_single_value!("min-width", MinWidthProperty, Val, val, min_size.width);
style_property! {
    #[doc = " Specify element minimum width by providing value to `Style.min_width`:"]
    #[doc = " ```css"]
    #[doc = " min-width: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Size Constraints -->"]
    MinWidthProperty("min-width") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.min_width != value {
                style.min_width = *value;
            }
        };
    }
}
// impl_style_single_value!("min-height", MinHeightProperty, Val, val, min_size.height);
style_property! {
    #[doc = " Specify element minimum height by providing value to `Style.min_height`:"]
    #[doc = " ```css"]
    #[doc = " min-height: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Size Constraints -->"]
    MinHeightProperty("min-height") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.min_height != value {
                style.min_height = *value;
            }
        };
    }
}

// impl_style_single_value!("max-width", MaxWidthProperty, Val, val, max_size.width);
style_property! {
    #[doc = " Specify element maximum width by providing value to `Style.max_width`:"]
    #[doc = " ```css"]
    #[doc = " max-width: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Size Constraints -->"]
    MaxWidthProperty("max-width") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.max_width != value {
                style.max_width = *value;
            }
        };
    }
}
// impl_style_single_value!("max-height", MaxHeightProperty, Val, val, max_size.height);
style_property! {
    #[doc = " Specify element maximum height by providing value to `Style.max_height`:"]
    #[doc = " ```css"]
    #[doc = " max-height: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Size Constraints -->"]
    MaxHeightProperty("max-height") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.max_height != value {
                style.max_height = *value;
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
    #[doc = " <!-- @property-category=Size Constraints -->"]
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
