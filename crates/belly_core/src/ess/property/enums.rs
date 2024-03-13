use super::StyleProperty;
use super::StylePropertyMethods;
use super::StylePropertyToken;
use crate::ElementsError;
use bevy::prelude::*;

macro_rules! prop_to_enum {
    (@join $item1:literal,) => {
        $item1
    };
    (@join $item1:literal, $item2:literal,) => {
        concat!($item1, "|", $item2)
    };
    (@join $item:literal, $($rest:literal,)+) => {
        concat!($item, "|", prop_to_enum!(@join $($rest,)+))
    };
    ($typ:ty, $($prop:literal => $variant:expr,)+) => {
        impl TryFrom<&StyleProperty> for $typ {
            type Error = ElementsError;
            fn try_from(value: &StyleProperty) -> Result<$typ, ElementsError> {
                let ts = prop_to_enum!(@join $($prop,)+);
                let Some(StylePropertyToken::Identifier(ident)) = value.first() else {
                    return Err(ElementsError::InvalidPropertyValue(format!(
                        "Expected {}, got `{}`", ts, value.to_string()
                    )))
                };
                use $typ::*;
                match ident.as_str() {
                    $($prop => return Ok($variant),)+
                    ident => Err(ElementsError::InvalidPropertyValue(format!(
                        "Expected {}, got `{}`", ts, ident
                    )))
                }
            }
        }
    };
}

prop_to_enum! { Display,
    "none" => None,
    "flex" => Flex,
    "grid" => Grid,
}

prop_to_enum! { PositionType,
    "absolute" => Absolute,
    "relative" => Relative,
}

prop_to_enum! { Direction,
    "inherit" => Inherit,
    "ltr" => LeftToRight,
    "rtl" => RightToLeft,
}

prop_to_enum! { FlexDirection,
    "row" => Row,
    "column" => Column,
    "row-reverse" => RowReverse,
    "column-reverse" => ColumnReverse,
}

prop_to_enum! { FlexWrap,
    "no-wrap" => NoWrap,
    "wrap" => Wrap,
    "wrap-reverse" => WrapReverse,
}

prop_to_enum! { AlignItems,
    "default" => Default,
    "start" => Start,
    "end" => End,
    "flex-start" => FlexStart,
    "flex-end" => FlexEnd,
    "center" => Center,
    "baseline" => Baseline,
    "stretch" => Stretch,
}

prop_to_enum! { AlignContent,
    "default" => Default,
    "start" => Start,
    "end" => End,
    "flex-start" => FlexStart,
    "flex-end" => FlexEnd,
    "center" => Center,
    "stretch" => Stretch,
    "space-between" => SpaceBetween,
    "space-around" => SpaceAround,
}

prop_to_enum! { JustifyContent,
    "default" => Default,
    "start" => Start,
    "end" => End,
    "flex-start" => FlexStart,
    "flex-end" => FlexEnd,
    "center" => Center,
    "space-between" => SpaceBetween,
    "space-around" => SpaceAround,
    "space-evenly" => SpaceEvenly,
}

prop_to_enum! { JustifyItems,
    "default" => Default,
    "start" => Start,
    "end" => End,
    "center" => Center,
    "baseline" => Baseline,
    "stretch" => Stretch,
}

prop_to_enum! { JustifySelf,
    "auto" => Auto,
    "start" => Start,
    "end" => End,
    "center" => Center,
    "baseline" => Baseline,
    "stretch" => Stretch,
}

prop_to_enum! { AlignSelf,
    "auto" => Auto,
    "flex-start" => FlexStart,
    "flex-end" => FlexEnd,
    "center" => Center,
    "baseline" => Baseline,
    "stretch" => Stretch,
}

prop_to_enum! { GridAutoFlow,
    "row" => Row,
    "column" => Column,
    "row-dense" => RowDense,
    "column-dense" => ColumnDense,
}
