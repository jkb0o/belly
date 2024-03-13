use super::parse;
use crate::build::StyleProperty;
use crate::ess::{PropertyParser, StylePropertyToken};
use crate::style_property;
use crate::ElementsError;
use bevy::prelude::*;
use smallvec::smallvec;

style_property! {
    #[doc = " Controls whether automatically placed grid items are placed row-wise or"]
    #[doc = " column-wise. And whether the sparse or dense packing algorithm is used."]
    #[doc = " "]
    #[doc = " Only affect Grid layouts."]
    #[doc = " "]
    #[doc = " The `dense` packing algorithm attempts to fill in holes earlier in the grid,"]
    #[doc = " if smaller items come up later. This may cause items to appear out-of-order,"]
    #[doc = " when doing so would fill in holes left by larger items."]
    #[doc = " "]
    #[doc = " ```css"]
    #[doc = " grid-auto-flow: row-dense;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " Supported values:"]
    #[doc = " `row` - items are placed by filling each row in turn, adding new rows as necessary"]
    #[doc = " `column` - items are placed by filling each column in turn, adding new columns as necessary."]
    #[doc = " `row-dense` - combines `row` with the dense packing algorithm."]
    #[doc = " `column-dense` - combines `column` with the dense packing algorithm."]
    #[doc = " "]
    #[doc = " <https://developer.mozilla.org/en-US/docs/Web/CSS/grid-auto-flow>"]
    #[doc = " <!-- @property-category=Grid -->"]
    GridAutoFlowProperty("grid-auto-flow") {
        Default = "row";
        Item = GridAutoFlow;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::IdentifierParser<GridAutoFlow>;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.grid_auto_flow != value {
                style.grid_auto_flow = *value;
            }
        };
    }
}

pub fn grid_track(token: &StylePropertyToken) -> Result<GridTrack, ElementsError> {
    match token {
        StylePropertyToken::Dimension(val, dim) if dim.as_str() == "px" => {
            Ok(GridTrack::px(val.to_float()))
        }
        StylePropertyToken::Dimension(val, dim) if dim.as_str() == "fr" => {
            Ok(GridTrack::fr(val.to_float()))
        }
        StylePropertyToken::Percentage(val) => Ok(GridTrack::percent(val.to_float())),
        StylePropertyToken::Identifier(ident) if ident.as_str() == "auto" => Ok(GridTrack::auto()),
        StylePropertyToken::Identifier(ident) if ident.as_str() == "min-content" => {
            Ok(GridTrack::min_content())
        }
        StylePropertyToken::Identifier(ident) if ident.as_str() == "max-content" => {
            Ok(GridTrack::max_content())
        }
        StylePropertyToken::Function(func) => match func.name.as_str() {
            "flex" => {
                if func.args.len() != 1 {
                    return Err(ElementsError::InvalidPropertyValue(format!(
                        "flex($num) supports only single argument"
                    )));
                }
                let StylePropertyToken::Number(val) = func.args[0] else {
                    return Err(ElementsError::InvalidPropertyValue(format!(
                        "flex($num) supports only single argument"
                    )));
                };
                Ok(GridTrack::flex(val.to_float()))
            }
            "fit-content" => {
                if func.args.len() != 1 {
                    return Err(ElementsError::InvalidPropertyValue(format!(
                        "fit-content($val) supports only single px or % argument"
                    )));
                }
                match &func.args[0] {
                    StylePropertyToken::Dimension(val, dim) if dim.as_str() == "px" => {
                        Ok(GridTrack::fit_content_px(val.to_float()))
                    }
                    StylePropertyToken::Percentage(val) => {
                        Ok(GridTrack::fit_content_percent(val.to_float()))
                    }
                    token => {
                        return Err(ElementsError::InvalidPropertyValue(format!(
                            "fit-content($val) supports only single px or % argument, got: `{}`",
                            token.to_string()
                        )))
                    }
                }
            }
            "minmax" => {
                if func.args.len() != 2 {
                    return Err(ElementsError::InvalidPropertyValue(format!(
                        "minmax(a, b) only supports exactly two arguments"
                    )));
                }
                let min_sizing = match &func.args[0] {
                    StylePropertyToken::Dimension(val, dim) if dim.as_str() == "px" => {
                        MinTrackSizingFunction::Px(val.to_float())
                    }
                    StylePropertyToken::Percentage(val) => {
                        MinTrackSizingFunction::Percent(val.to_float())
                    }
                    StylePropertyToken::Identifier(ident) if ident.as_str() == "auto" => {
                        MinTrackSizingFunction::Auto
                    }
                    StylePropertyToken::Identifier(ident) if ident.as_str() == "min-content" => {
                        MinTrackSizingFunction::MinContent
                    }
                    StylePropertyToken::Identifier(ident) if ident.as_str() == "max-content" => {
                        MinTrackSizingFunction::MaxContent
                    }
                    token => {
                        return Err(ElementsError::InvalidPropertyValue(format!(
                            "invalid value for the first argument of minmax(a, b): `{}`",
                            token.to_string(),
                        )))
                    }
                };
                let max_sizing = match &func.args[1] {
                    StylePropertyToken::Dimension(val, dim) if dim.as_str() == "px" => {
                        MaxTrackSizingFunction::Px(val.to_float())
                    }
                    StylePropertyToken::Dimension(val, dim) if dim.as_str() == "fr" => {
                        MaxTrackSizingFunction::Fraction(val.to_float())
                    }
                    StylePropertyToken::Percentage(val) => {
                        MaxTrackSizingFunction::Percent(val.to_float())
                    }
                    StylePropertyToken::Identifier(ident) if ident.as_str() == "auto" => {
                        MaxTrackSizingFunction::Auto
                    }
                    StylePropertyToken::Identifier(ident) if ident.as_str() == "min-content" => {
                        MaxTrackSizingFunction::MinContent
                    }
                    StylePropertyToken::Identifier(ident) if ident.as_str() == "max-content" => {
                        MaxTrackSizingFunction::MaxContent
                    }
                    StylePropertyToken::Identifier(ident) if ident.as_str() == "max-content" => {
                        MaxTrackSizingFunction::MaxContent
                    }
                    StylePropertyToken::Function(func) if func.name.as_str() == "fit-content" => {
                        if func.args.len() != 1 {
                            return Err(ElementsError::InvalidPropertyValue(format!(
                                "fit-content($val) only supports exactly single argument"
                            )));
                        }
                        match &func.args[0] {
                            StylePropertyToken::Dimension(val, dim) if dim == "px" => {
                                MaxTrackSizingFunction::FitContentPx(val.to_float())
                            }
                            StylePropertyToken::Percentage(val) => {
                                MaxTrackSizingFunction::FitContentPercent(val.to_float())
                            }
                            token => {
                                return Err(ElementsError::InvalidPropertyValue(format!(
                                    "unsupported argument for fit-content($val): `{}`",
                                    token.to_string(),
                                )))
                            }
                        }
                    }
                    token => {
                        return Err(ElementsError::InvalidPropertyValue(format!(
                            "invalid value for the second argument of minmax(a, b): `{}`",
                            token.to_string(),
                        )))
                    }
                };
                Ok(GridTrack::minmax(min_sizing, max_sizing))
            }
            _ => {
                return Err(ElementsError::InvalidPropertyValue(format!(
                    "unsupported value for $gridtracks property: `{}`",
                    token.to_string(),
                )))
            }
        },
        token => {
            return Err(ElementsError::InvalidPropertyValue(format!(
                "invalid value for $gridtracks property: `{}`",
                token.to_string()
            )))
        }
    }
}

pub fn grid_tracks(prop: &StyleProperty) -> Result<Vec<GridTrack>, ElementsError> {
    let mut result = vec![];
    if prop.len() == 1 && prop[0].is_ident("none") {
        return Ok(vec![]);
    }
    for token in prop.iter() {
        result.push(grid_track(token)?)
    }
    Ok(result)
}

/// <!-- @property-type=$gridtracks
pub struct GridTrackParser;
impl PropertyParser<Vec<GridTrack>> for GridTrackParser {
    fn parse(value: &StyleProperty) -> Result<Vec<GridTrack>, ElementsError> {
        grid_tracks(value)
    }
}

style_property! {
    #[doc = " Defines the size of implicitly created rows. Rows are created implicitly"]
    #[doc = " when grid items are given explicit placements that are out of bounds"]
    #[doc = " of the rows explicitly created using `grid_template_rows`."]
    #[doc = " "]
    #[doc = " <https://developer.mozilla.org/en-US/docs/Web/CSS/grid-template-rows>"]
    #[doc = " <!-- @property-type=$gridtracks -->"]
    #[doc = " <!-- @property-category=Grid -->"]
    GridAutoRowsProperty("grid-auto-rows") {
        Default = "none";
        Item = Vec<GridTrack>;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = GridTrackParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.grid_auto_rows != value {
                style.grid_auto_rows = value.clone();
            }
        };
    }
}

style_property! {
    #[doc = " Defines the size of implicitly created columns. Columns are created implicitly"]
    #[doc = " when grid items are given explicit placements that are out of bounds"]
    #[doc = " of the columns explicitly created using `grid_template_columms`."]
    #[doc = " "]
    #[doc = " <https://developer.mozilla.org/en-US/docs/Web/CSS/grid-template-columns>"]
    #[doc = " <!-- @property-type=$gridtracks -->"]
    #[doc = " <!-- @property-category=Grid -->"]
    GridAutoColumnsProperty("grid-auto-columns") {
        Default = "none";
        Item = Vec<GridTrack>;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = GridTrackParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.grid_auto_columns != value {
                style.grid_auto_columns = value.clone();
            }
        };
    }
}

pub fn grid_tracks_repeated(prop: &StyleProperty) -> Result<Vec<RepeatedGridTrack>, ElementsError> {
    if prop.len() == 1 && prop[0].is_ident("none") {
        return Ok(vec![]);
    }
    let mut result = vec![];
    for token in prop.iter() {
        result.push(match token {
        StylePropertyToken::Function(func) if func.name.as_str() == "repeat" => {
            if func.args.len() != 2 {
                return Err(ElementsError::InvalidPropertyValue(format!(
                    "repeat() func accepts exactly two arguments",
                )))
            }
            let repeat_func = match &func.args[0] {
                StylePropertyToken::Number(num) => {
                    GridTrackRepetition::Count(num.to_int() as u16)
                },
                StylePropertyToken::Identifier(ident) if ident.as_str() == "auto-fill" => {
                    GridTrackRepetition::AutoFill
                },
                StylePropertyToken::Identifier(ident) if ident.as_str() == "auto-fit" => {
                    GridTrackRepetition::AutoFit
                },
                token => return Err(ElementsError::InvalidPropertyValue(format!(
                    "invalid first argument to repeat() function: expected $num|auto-fill|auto-fit, got `{}`",
                    token.to_string()
                )))
            };
            let repeat_args = grid_tracks(&StyleProperty(match &func.args[1] {
                StylePropertyToken::Tokens(tokens) => tokens.clone().into(),
                token => smallvec![token.clone()]
            }))?;
            RepeatedGridTrack::repeat_many(repeat_func, repeat_args)
        },
        token => RepeatedGridTrack::repeat_many(1, grid_track(token)?)
    })
    }
    Ok(result)
}

/// <!-- @property-type=$gridtracksrepeated
pub struct RepeatedGridTrackParser;
impl PropertyParser<Vec<RepeatedGridTrack>> for RepeatedGridTrackParser {
    fn parse(value: &StyleProperty) -> Result<Vec<RepeatedGridTrack>, ElementsError> {
        grid_tracks_repeated(value)
    }
}

style_property! {
    #[doc = " Defines the number of rows a grid has and the sizes of those rows."]
    #[doc = " If grid items are given explicit placements then more rows may"]
    #[doc = " be implicitly generated by items that are placed out of bounds."]
    #[doc = " The sizes of those rows are controlled by `grid-auto-rows` property."]
    #[doc = " "]
    #[doc = " <https://developer.mozilla.org/en-US/docs/Web/CSS/grid-template-rows>"]
    #[doc = " <!-- @property-type=$gridtracksrepeated -->"]
    #[doc = " <!-- @property-category=Grid -->"]
    GridTemplateRowsProperty("grid-template-rows") {
        Default = "none";
        Item = Vec<RepeatedGridTrack>;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = RepeatedGridTrackParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.grid_template_rows != value {
                style.grid_template_rows = value.clone();
            }
        };
    }
}

style_property! {
    #[doc = " Defines the number of columns a grid has and the sizes of those columns."]
    #[doc = " If grid items are given explicit placements then more columns may"]
    #[doc = " be implicitly generated by items that are placed out of bounds."]
    #[doc = " The sizes of those columns are controlled by `grid-auto-columns` property."]
    #[doc = " "]
    #[doc = " <https://developer.mozilla.org/en-US/docs/Web/CSS/grid-template-columns>"]
    #[doc = " <!-- @property-type=$gridtracksrepeated -->"]
    #[doc = " <!-- @property-category=Grid -->"]
    GridTemplateColumnsProperty("grid-template-columns") {
        Default = "none";
        Item = Vec<RepeatedGridTrack>;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = RepeatedGridTrackParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.grid_template_columns != value {
                style.grid_template_columns = value.clone();
                // style.grid_auto_flow
            }
        };
    }
}

pub fn grid_placement(prop: &StyleProperty) -> Result<GridPlacement, ElementsError> {
    let mut placement = GridPlacement::default();
    let mut parsing_start = true;
    let mut parsed_value = None;
    let mut parsed_is_span = false;
    for token in prop.iter() {
        match token {
            StylePropertyToken::Number(num) if parsed_value.is_none() => {
                parsed_value = Some(num.to_int());
            }
            StylePropertyToken::Identifier(ident) if ident == "span" => {
                parsed_is_span = true;
            }
            StylePropertyToken::Identifier(ident)
                if ident == "auto" && !parsed_is_span && parsed_value.is_none() =>
            {
                parsing_start = false;
            }
            StylePropertyToken::Slash if parsing_start => {
                if parsed_value.is_some() {
                    if parsed_is_span {
                        placement = placement.set_span(parsed_value.unwrap() as u16);
                    } else {
                        placement = placement.set_start(parsed_value.unwrap() as i16);
                    }
                }
                parsing_start = false;
                parsed_value = None;
                parsed_is_span = false;
            }
            _ => {
                return Err(ElementsError::InvalidPropertyValue(format!(
                    "Invalid format for GridPlacement value"
                )))
            }
        };
    }
    match (parsing_start, parsed_value, parsed_is_span) {
        (_, Some(num), true) => placement = placement.set_span(num as u16),
        (true, Some(num), false) => placement = placement.set_start(num as i16),
        (false, Some(num), false) => placement = placement.set_end(num as i16),
        _ => (),
    };

    Ok(placement)
}

/// <!-- @property-type=$gridplacement
pub struct GridPlacementParser;
impl PropertyParser<GridPlacement> for GridPlacementParser {
    fn parse(value: &StyleProperty) -> Result<GridPlacement, ElementsError> {
        grid_placement(value)
    }
}

style_property! {
    #[doc = " The row in which a grid item starts and how many rows it spans."]
    #[doc = " ```css"]
    #[doc = " grid-row: span 2 / 7;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " <https://developer.mozilla.org/en-US/docs/Web/CSS/grid-row>"]
    #[doc = " <!-- @property-type=$gridplacement -->"]
    #[doc = " <!-- @property-category=Grid -->"]
    GridRowProperty("grid-row") {
        Default = "span 1";
        Item = GridPlacement;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = GridPlacementParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.grid_row != value {
                style.grid_row = *value;
            }
        };
    }
}

style_property! {
    #[doc = " The row in which a grid item starts and how many rows it spans."]
    #[doc = " ```css"]
    #[doc = " grid-column: 2 / span 7;"]
    #[doc = " ```"]
    #[doc = " "]
    #[doc = " <https://developer.mozilla.org/en-US/docs/Web/CSS/grid-row>"]
    #[doc = " <!-- @property-type=$gridplacement -->"]
    #[doc = " <!-- @property-category=Grid -->"]
    GridColumnProperty("grid-column") {
        Default = "span 1";
        Item = GridPlacement;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = GridPlacementParser;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.grid_column != value {
                style.grid_column = *value;
            }
        };
    }
}

style_property! {
    #[doc = " For Flexbox items:"]
    #[doc = "   - This property has no effect. See `justify-content` for main-axis alignment of flex items."]
    #[doc = " For CSS Grid items:"]
    #[doc = "   - Controls inline (horizontal) axis alignment of a grid item within it's grid area."]
    #[doc =  ""]
    #[doc = " If set to `auto`, alignment is inherited from the value of `justify-items`] set on the parent node."]
    #[doc = " "]
    #[doc = " <https://developer.mozilla.org/en-US/docs/Web/CSS/justify-items>"]
    #[doc = " "]
    #[doc = " Suported values:"]
    #[doc = " - `auto`: Use the parent node's `align-items` value to determine"]
    #[doc = "   how this item should be aligned."]
    #[doc = " - `start`: This item will be aligned with the start of the axis."]
    #[doc = " - `end`: This item will be aligned with the end of the axis."]
    #[doc = " - `center`: This item will be aligned at the center."]
    #[doc = " - `baseline`: This item will be aligned at the baseline."]
    #[doc = " - `stretch`: This item will be stretched across the whole cross axis."]
    #[doc = " <!-- @property-category=Grid -->"]
    JustifySelfProperty("justify-self") {
        Default = "auto";
        Item = JustifySelf;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::IdentifierParser<JustifySelf>;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.justify_self != value {
                style.justify_self = *value;
            }
        };
    }
}

style_property! {
    #[doc = " For Flexbox containers:"]
    #[doc = " - This property has no effect. See `justify-content` for main-axis alignment of flex items."]
    #[doc = " For CSS Grid containers:"]
    #[doc = " - Sets default inline (horizontal) axis alignment of child items within their grid areas"]
    #[doc = " "]
    #[doc = " This value is overriden `justify-self` on the child node is set."]
    #[doc = " "]
    #[doc = " <https://developer.mozilla.org/en-US/docs/Web/CSS/justify-items>"]
    #[doc = " "]
    #[doc = " Supported values:"]
    #[doc = " - `default`: The items are packed in their default position as"]
    #[doc = "   if no alignment was applied"]
    #[doc = " - `start`: Items are packed towards the start of the axis."]
    #[doc = " - `end`: Items are packed towards the end of the axis."]
    #[doc = " - `center`: Items are aligned at the center."]
    #[doc = " - `baseline`: Items are aligned at the baseline."]
    #[doc = " - `stretch`: Items are stretched across the whole cross axis."]
    #[doc = " <!-- @property-category=Grid -->"]
    JustifyItemsProperty("justify-items") {
        Default = "default";
        Item = JustifyItems;
        Components = &'static mut Style;
        Filters = With<Node>;
        Parser = parse::IdentifierParser<JustifyItems>;
        Apply = |value, style, _assets, _commands, _entity| {
            if &style.justify_items != value {
                style.justify_items = *value;
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn parse_grid_placement() {
        for prop in &["span 2", "2 span"] {
            let p = StyleProperty::from_str(prop).unwrap();
            let g = GridPlacementParser::parse(&p).unwrap();
            assert_eq!(g, GridPlacement::span(2));
        }

        let p = StyleProperty::from_str("2 / 4").unwrap();
        let g = GridPlacementParser::parse(&p).unwrap();
        assert_eq!(g, GridPlacement::default().set_start(2).set_end(4));

        let p = StyleProperty::from_str("2 / span 2").unwrap();
        let g = GridPlacementParser::parse(&p).unwrap();
        assert_eq!(g, GridPlacement::start_span(2, 2));

        let p = StyleProperty::from_str("2 span / auto").unwrap();
        let g = GridPlacementParser::parse(&p).unwrap();
        assert_eq!(g, GridPlacement::span(2));
    }

    #[test]
    fn parse_grid_track_repeated() {
        let p = StyleProperty::from_str("auto flex(1.0) 20px").unwrap();
        let g = RepeatedGridTrackParser::parse(&p).unwrap();
        let r: Vec<RepeatedGridTrack> =
            vec![GridTrack::auto(), GridTrack::flex(1.0), GridTrack::px(20.)];
        assert_eq!(g, r);

        let p = StyleProperty::from_str("min-content flex(1.0)").unwrap();
        let g = RepeatedGridTrackParser::parse(&p).unwrap();
        let r: Vec<RepeatedGridTrack> = vec![GridTrack::min_content(), GridTrack::flex(1.0)];
        assert_eq!(g, r);

        let p = StyleProperty::from_str("repeat(4, flex(1.0)").unwrap();
        let g = RepeatedGridTrackParser::parse(&p).unwrap();
        let r: Vec<RepeatedGridTrack> = RepeatedGridTrack::flex(4, 1.0);
        assert_eq!(g, r);

        let p = StyleProperty::from_str("auto auto 1fr").unwrap();
        let g = RepeatedGridTrackParser::parse(&p).unwrap();
        let r: Vec<RepeatedGridTrack> =
            vec![GridTrack::auto(), GridTrack::auto(), GridTrack::fr(1.0)];
        assert_eq!(g, r);

        // This test failes. I'm not sure are this tracks the same or not
        // https://discord.com/channels/691052431525675048/885021580353237032/1141424049084436642
        //
        // let p0 = StyleProperty::from_str("auto auto 1fr").unwrap();
        // let p1 = StyleProperty::from_str("repeat(1, auto auto 1fr)").unwrap();
        // let g0 = RepeatedGridTrackParser::parse(&p0).unwrap();
        // let g1 = RepeatedGridTrackParser::parse(&p1).unwrap();
        // assert_eq!(g0, g1);

        // left:
        // [
        //  RepeatedGridTrack { repetition: Count(1), tracks: [GridTrack { min_sizing_function: Auto, max_sizing_function: Auto }] },
        //  RepeatedGridTrack { repetition: Count(1), tracks: [GridTrack { min_sizing_function: Auto, max_sizing_function: Auto }] },
        //  RepeatedGridTrack { repetition: Count(1), tracks: [GridTrack { min_sizing_function: Auto, max_sizing_function: Fraction(1.0) }] }
        // ]
        // right:
        // [
        //  RepeatedGridTrack { repetition: Count(1), tracks: [
        //      GridTrack { min_sizing_function: Auto, max_sizing_function: Auto },
        //      GridTrack { min_sizing_function: Auto, max_sizing_function: Auto },
        //      GridTrack { min_sizing_function: Auto, max_sizing_function: Fraction(1.0) }
        //  ]
        // }]
    }
}
