use super::parse;
use crate::compound_style_property;
use crate::ess::ToRectMap;
use crate::style_property;
use bevy::prelude::*;

compound_style_property! {
    #[doc = " Specify element margin by providing values to `Style.margin`:"]
    #[doc = " ```css"]
    #[doc = " margin: 2px 20% 10px auto;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Margins are used to create space around elements, outside of"]
    #[doc = " any defined borders."]
    #[doc = " <!-- @property-type=$rect -->"]
    #[doc = " <!-- @property-category=Spacing -->"]
    MarginProperty("margin", value) {
        let rect = UiRect::try_from(value)?;
        Ok(rect.to_rect_map("margin-"))
    }
}

style_property! {
    #[doc = " Specify element left margin by providing value to `Style.margin.left`:"]
    #[doc = " ```css"]
    #[doc = " margin-left: 5px;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Margins are used to create space around elements, outside of"]
    #[doc = " any defined borders."]
    #[doc = " <!-- @property-category=Spacing -->"]
    MarginLeftProperty("margin-left") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.margin.left != value {
                style.margin.left = *value;
            }
        };
    }
}
// impl_style_single_value!("margin-right", MarginRightProperty, Val, val, margin.right);
style_property! {
    #[doc = " Specify element right margin by providing value to `Style.margin.right`:"]
    #[doc = " ```css"]
    #[doc = " margin-right: 5px;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Margins are used to create space around elements, outside of"]
    #[doc = " any defined borders."]
    #[doc = " <!-- @property-category=Spacing -->"]
    MarginRightProperty("margin-right") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.margin.right != value {
                style.margin.right = *value;
            }
        };
    }
}
// impl_style_single_value!("margin-top", MarginTopProperty, Val, val, margin.top);
style_property! {
    #[doc = " Specify element top margin by providing value to `Style.margin.top`:"]
    #[doc = " ```css"]
    #[doc = " margin-top: 5px;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Margins are used to create space around elements, outside of"]
    #[doc = " any defined borders."]
    #[doc = " <!-- @property-category=Spacing -->"]
    MarginTopProperty("margin-top") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.margin.top != value {
                style.margin.top = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element bottom margin by providing value to `Style.margin.bottom`:"]
    #[doc = " ```css"]
    #[doc = " margin-bottom: 5px;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Margins are used to create space around elements, outside of"]
    #[doc = " any defined borders."]
    #[doc = " <!-- @property-category=Spacing -->"]
    MarginBottomProperty("margin-bottom") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.margin.bottom != value {
                style.margin.bottom = *value;
            }
        };
    }
}

compound_style_property! {
    #[doc = " Specify element padding by providing values to `Style.padding`:"]
    #[doc = " ```css"]
    #[doc = " padding: 2px 20% 10px auto;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Padding is used to create space around an element's content, inside of"]
    #[doc = " any defined borders."]
    #[doc = " <!-- @property-type=$rect -->"]
    #[doc = " <!-- @property-category=Spacing -->"]
    PaddingProperty("padding", value) {
        let rect = UiRect::try_from(value)?;
        Ok(rect.to_rect_map("padding-"))
    }
}

style_property! {
    #[doc = " Specify element left padding by providing value to `Style.padding.left`:"]
    #[doc = " ```css"]
    #[doc = " padding-left: 5px;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Padding is used to create space around an element's content, inside of"]
    #[doc = " any defined borders."]
    #[doc = " <!-- @property-category=Spacing -->"]
    PaddingLeftProperty("padding-left") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.padding.left != value {
                style.padding.left = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element right padding by providing value to `Style.padding.right`:"]
    #[doc = " ```css"]
    #[doc = " padding-right: 5px;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Padding is used to create space around an element's content, inside of"]
    #[doc = " any defined borders."]
    #[doc = " <!-- @property-category=Spacing -->"]
    PaddingRightProperty("padding-right") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.padding.right != value {
                style.padding.right = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element top padding by providing value to `Style.padding.top`:"]
    #[doc = " ```css"]
    #[doc = " padding-top: 5px;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Padding is used to create space around an element's content, inside of"]
    #[doc = " any defined borders."]
    #[doc = " <!-- @property-category=Spacing -->"]
    PaddingTopProperty("padding-top") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.padding.top != value {
                style.padding.top = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element bottom padding by providing value to `Style.padding.bottom`:"]
    #[doc = " ```css"]
    #[doc = " padding-bottom: 5px;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Padding is used to create space around an element's content, inside of"]
    #[doc = " any defined borders."]
    #[doc = " <!-- @property-category=Spacing -->"]
    PaddingBottomProperty("padding-bottom") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.padding.bottom != value {
                style.padding.bottom = *value;
            }
        };
    }
}

compound_style_property! {
    #[doc = " Specify element border width by providing values to `Style.border`:"]
    #[doc = " ```css"]
    #[doc = " border-width: 2px 20% 10px auto;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " The `border-width` property specifies the width of the four borders."]
    #[doc = " <!-- @property-type=$rect -->"]
    #[doc = " <!-- @property-category=Spacing -->"]
    BorderProperty("border-width", value) {
        let rect = UiRect::try_from(value)?;
        Ok(rect.to_rect_map("border-width-"))
    }
}

style_property! {
    #[doc = " Specify element left border width by providing value to `Style.border.left`:"]
    #[doc = " ```css"]
    #[doc = " border-width-left: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Spacing -->"]
    BorderLeftProperty("border-width-left") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.border.left != value {
                style.border.left = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element right border width by providing value to `Style.border.right`:"]
    #[doc = " ```css"]
    #[doc = " border-width-right: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    #[doc = " <!-- @property-category=Spacing -->"]
    BorderRightProperty("border-width-right") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.border.right != value {
                style.border.right = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element top border width by providing value to `Style.border.top`:"]
    #[doc = " ```css"]
    #[doc = " border-width-top: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Spacing -->"]
    BorderTopProperty("border-width-top") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.border.top != value {
                style.border.top = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element bottom border width by providing value to `Style.border.bottom`:"]
    #[doc = " ```css"]
    #[doc = " border-width-bottom: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Spacing -->"]
    BorderBottomProperty("border-width-bottom") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.border.bottom != value {
                style.border.bottom = *value;
            }
        };
    }
}
