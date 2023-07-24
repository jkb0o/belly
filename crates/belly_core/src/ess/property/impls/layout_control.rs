use super::parse;
use crate::compound_style_property;
use crate::ess::ToRectMap;
use crate::style_property;
use bevy::prelude::*;

style_property! {
    #[doc = " Specify how an element is positioned in a document acording to the `top`,"]
    #[doc = " `right`, `bottom`, and `left` by providing value to `style_type`:"]
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
    #[doc = " <!-- @property-category=Layout Control -->"]
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
    #[doc = " Specify element position by providing values to `style`:"]
    #[doc = " ```css"]
    #[doc = " position: 2px 20% 10px auto;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-type=$rect -->"]
    #[doc = " <!-- @property-category=Layout Control -->"]
    PositionProperty("position", value) {
        let rect = UiRect::try_from(value)?;
        Ok(rect.to_rect_map(""))
    }
}

style_property! {
    #[doc = " Specify element left position by providing value to `style.left`:"]
    #[doc = " ```css"]
    #[doc = " left: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Layout Control Positioning -->"]
    LeftProperty("left") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.left != value {
                style.left = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element right position by providing value to `style.right`:"]
    #[doc = " ```css"]
    #[doc = " right: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Layout Control Positioning -->"]
    RightProperty("right") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.right != value {
                style.right = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element top position by providing value to `style.top`:"]
    #[doc = " ```css"]
    #[doc = " top: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Layout Control Positioning -->"]
    TopProperty("top") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.top != value {
                style.top = *value;
            }
        };
    }
}

style_property! {
    #[doc = " Specify element bottom position by providing value to `style.bottom`:"]
    #[doc = " ```css"]
    #[doc = " bottom: 5px;"]
    #[doc = " ```"]
    #[doc = " <!-- @property-category=Layout Control Positioning -->"]
    BottomProperty("bottom") {
        Default = "undefined";
        Item = Val;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::ValParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.bottom != value {
                style.bottom = *value;
            }
        };
    }
}

// todo!(add back Overflow)
// style_property! {
//     #[doc = " TODO: add OverflowProperty descripion"]
//     #[doc = " <!-- @property-category=Layout Control -->"]
//     OverflowProperty("overflow") {
//         Default = "visible";
//         Item = Overflow;
//         Components = &'static mut Style;
//         Filters = With<Node>;
//         Parser = parse::IdentifierParser<Overflow>;
//         Apply = |value, style, _assets, _commands, _entity| {
//             if &style.overflow != value {
//                 style.overflow = *value;
//             }
//         };
//     }
// }

style_property! {
    #[doc = " Specify element display by providing value to `Style.display`:"]
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
    #[doc = " <!-- @property-category=Layout Control -->"]
    DisplayProperty("display") {
        Default = "flex";
        Item = Display;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::IdentifierParser<Display>;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.display != value {
                info!("set display = {value:?}");
                style.display = *value;
            }
        };
    }
}
