use super::parse;
use crate::compound_style_property;
use crate::ess::ToRectMap;
use crate::style_property;
use bevy::prelude::*;

style_property! {
    #[doc = " Specify how an element is positioned in a document acording to the `top`,"]
    #[doc = " `right`, `bottom`, and `left` by providing value to `Style.position_type`:"]
    #[doc = " ```css"]
    #[doc = " position-type: absolute;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Supported values:"]
    #[doc = " - `absolute`: the element is removed from the normal document flow, and no"]
    #[doc = "   space is created for the element in the page layout. It is positioned relative"]
    #[doc = "   to its closest positioned ancestor. Its final position is determined by the"]
    #[doc = "   values of `top`, `right`, `bottom`, and `left`."]
    #[doc = " - `relative`: the element is positioned according to the normal flow of the document"]
    #[doc = "   and then offset *relative* to itself based on the values of `top`, `right`, `bottom`"]
    #[doc = "   and left. The offset does not affect the position of any other elements."]
    #[doc = " "]
    #[doc = " Default: `relative`"]
    PositionTypeProperty("position-type") {
        Default = "relative";
        Item = PositionType;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::IdentifierParser<PositionType>;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.position_type != value {
                style.position_type = *value;
            }
        };
    }
}

compound_style_property! {
    #[doc = " Specify element position by providing values to `Style.position`"]
    #[doc = " using single `rect-shorthand` syntax:"]
    #[doc = " ```css"]
    #[doc = " position: 2px 20% 10px auto;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link rect-shorthand) -->"]
    PositionProperty("position", value) {
        let rect = UiRect::try_from(value)?;
        Ok(rect.to_rect_map(""))
    }
}

style_property! {
    #[doc = " Specify element left position by providing value to `Style.position.left`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " left: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    LeftProperty("left") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.position.left != value {
                style.position.left = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element right position by providing value to `Style.position.right`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " right: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    RightProperty("right") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.position.right != value {
                style.position.right = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element top position by providing value to `Style.position.top`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " top: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    TopProperty("top") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.position.top != value {
                style.position.top = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element bottom position by providing value to `Style.position.bottom`"]
    #[doc = " using `val` syntax:"]
    #[doc = " ```css"]
    #[doc = " bottom: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- (TODO: link val) -->"]
    BottomProperty("bottom") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.position.bottom != value {
                style.position.bottom = *value;
            }
        };
    }
}

style_property! {
    #[doc = " TODO: add OverflowProperty descripion"]
    OverflowProperty("overflow") {
        Default = "visible";
        Item = Overflow;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::IdentifierParser<Overflow>;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.overflow != value {
                style.overflow = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element display by providing value to"]
    #[doc = " `Style.display`:"]
    #[doc = " ```css"]
    #[doc = " display: none;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Supported values:"]
    #[doc = " - `none`: turns off the display of an element so that it has no effect"]
    #[doc = "   on layout (the document is rendered as though the element did not"]
    #[doc = "   exist). All descendant elements also have their display turned off."]
    #[doc = "   To have an element take up the space that it would normally take, but"]
    #[doc = "   without actually rendering anything"]
    #[doc = " - `flex`: display element according to the"]
    #[doc = "   [Flexbox](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Flexible_Box_Layout)."]
    #[doc = " "]
    #[doc = " Default: `flex`"]
    DisplayProperty("display") {
        Default = "flex";
        Item = Display;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::IdentifierParser<Display>;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.display != value {
                style.display = *value;
            }
        };
    }
}
