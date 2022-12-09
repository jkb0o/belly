use bevy::{
    prelude::{Color, Deref},
    ui::{UiRect, Val},
};
use cssparser::{BasicParseErrorKind, Token};
use smallvec::SmallVec;

use crate::ElementsError;

use super::colors;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub struct Number([u8; 4]);

impl Number {
    fn from_float(value: f32) -> Self {
        Number(value.to_le_bytes())
    }
    fn to_float(&self) -> f32 {
        f32::from_le_bytes(self.0)
    }
}

impl From<f32> for Number {
    fn from(v: f32) -> Self {
        Number::from_float(v)
    }
}
impl From<&f32> for Number {
    fn from(v: &f32) -> Self {
        Number::from_float(*v)
    }
}

impl From<Number> for f32 {
    fn from(v: Number) -> Self {
        v.to_float()
    }
}

impl From<&Number> for f32 {
    fn from(v: &Number) -> Self {
        v.to_float()
    }
}

/// A property value token which was parsed from a CSS rule.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub enum StylePropertyToken {
    /// A value which was parsed percent value, like `100%` or `73.23%`.
    Percentage(Number),
    /// A value which was parsed dimension value, like `10px` or `35em.
    ///
    /// Currently there is no distinction between [`length-values`](https://developer.mozilla.org/en-US/docs/Web/CSS/length).
    Dimension(Number),
    /// A numeric float value, like `31.1` or `43`.
    Number(Number),
    /// A plain identifier, like `none` or `center`.
    Identifier(String),
    /// A identifier prefixed by a hash, like `#001122`.
    Hash(String),
    /// A quoted string, like `"some value"`.
    String(String),
}

impl StylePropertyToken {
    fn to_string(&self) -> String {
        match self {
            StylePropertyToken::Percentage(v) => format!("{}%", v.to_float()),
            StylePropertyToken::Dimension(v) => format!("{}px", v.to_float()),
            StylePropertyToken::Number(v) => format!("{}", v.to_float()),
            StylePropertyToken::Identifier(v) => format!("{}", v),
            StylePropertyToken::Hash(v) => format!("#{}", v),
            StylePropertyToken::String(v) => format!("\"{}\"", v),
        }
    }

    fn val(&self) -> Result<Val, ElementsError> {
        match self {
            StylePropertyToken::Percentage(p) => Ok(Val::Percent(p.to_float())),
            StylePropertyToken::Dimension(d) => Ok(Val::Px(d.to_float())),
            StylePropertyToken::Identifier(i) if i == "auto" => Ok(Val::Auto),
            StylePropertyToken::Identifier(i) if i == "undefined" => Ok(Val::Undefined),
            _ => Err(ElementsError::InvalidPropertyValue(format!(
                "Can't treat `{}` as size value",
                self.to_string()
            ))),
        }
    }
}

impl<'i> TryFrom<Token<'i>> for StylePropertyToken {
    type Error = String;

    fn try_from(token: Token<'i>) -> Result<Self, Self::Error> {
        match token {
            Token::Ident(val) => Ok(Self::Identifier(val.to_string())),
            Token::Hash(val) => Ok(Self::Hash(val.to_string())),
            Token::IDHash(val) => Ok(Self::Hash(val.to_string())),
            Token::QuotedString(val) => Ok(Self::String(val.to_string())),
            Token::Number { value, .. } => Ok(Self::Number(value.into())),
            Token::Percentage { unit_value, .. } => {
                Ok(Self::Percentage((unit_value * 100.0).into()))
            }
            Token::Dimension { value, .. } => Ok(Self::Dimension(value.into())),
            token => Err(format!("Invalid token: {:?}", token)),
        }
    }
}

/// A list of [`PropertyToken`] which was parsed from a single property.
#[derive(Debug, Default, Clone, Deref, PartialEq, Eq, Hash)]
pub struct StyleProperty(pub(crate) SmallVec<[StylePropertyToken; 8]>);

impl From<&StyleProperty> for StyleProperty {
    fn from(v: &StyleProperty) -> Self {
        v.clone()
    }
}

impl StyleProperty {
    /// Tries to parses the current values as a single [`String`].
    pub fn string(&self) -> Option<String> {
        self.0.iter().find_map(|token| match token {
            StylePropertyToken::String(id) => {
                if id.is_empty() {
                    None
                } else {
                    Some(id.clone())
                }
            }
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Color`].
    ///
    /// Currently only [named colors](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color)
    /// and [hex-colors](https://developer.mozilla.org/en-US/docs/Web/CSS/hex-color) are supported.
    pub fn color(&self) -> Option<Color> {
        if self.0.len() == 1 {
            match &self.0[0] {
                StylePropertyToken::Identifier(name) => colors::parse_named_color(name.as_str()),
                StylePropertyToken::Hash(hash) => colors::parse_hex_color(hash.as_str()),
                _ => None,
            }
        } else {
            // TODO: Implement color function like rgba(255, 255, 255, 255)
            // https://developer.mozilla.org/en-US/docs/Web/CSS/color_value
            None
        }
    }

    /// Tries to parses the current values as a single identifier.
    pub fn identifier(&self) -> Option<&str> {
        self.0.iter().find_map(|token| match token {
            StylePropertyToken::Identifier(id) => {
                if id.is_empty() {
                    None
                } else {
                    Some(id.as_str())
                }
            }
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Val`].
    ///
    /// Only [`Percentage`](PropertyToken::Percentage) and [`Dimension`](PropertyToken::Dimension`) are considered valid values,
    /// where former is converted to [`Val::Percent`] and latter is converted to [`Val::Px`].
    pub fn val(&self) -> Option<Val> {
        self.0.iter().find_map(|token| match token {
            StylePropertyToken::Percentage(val) => Some(Val::Percent(val.into())),
            StylePropertyToken::Dimension(val) => Some(Val::Px(val.into())),
            StylePropertyToken::Identifier(val) if val.as_str() == "auto" => Some(Val::Auto),
            StylePropertyToken::Identifier(val) if val.as_str() == "undefined" => {
                Some(Val::Undefined)
            }
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`f32`].
    ///
    /// Only [`Percentage`](PropertyToken::Percentage), [`Dimension`](PropertyToken::Dimension`) and [`Number`](PropertyToken::Number`)
    /// are considered valid values.
    pub fn f32(&self) -> Option<f32> {
        self.0.iter().find_map(|token| match token {
            StylePropertyToken::Percentage(val)
            | StylePropertyToken::Dimension(val)
            | StylePropertyToken::Number(val) => Some(val.into()),
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Option<f32>`].
    ///
    /// This function is useful for properties where either a numeric value or a `none` value is expected.
    ///
    /// If a [`Option::None`] is returned, it means some invalid value was found.
    ///
    /// If there is a [`Percentage`](PropertyToken::Percentage), [`Dimension`](PropertyToken::Dimension`) or [`Number`](PropertyToken::Number`) token,
    /// a [`Option::Some`] with parsed [`Option<f32>`] is returned.
    /// If there is a identifier with a `none` value, then [`Option::Some`] with [`None`] is returned.
    pub fn option_f32(&self) -> Option<Option<f32>> {
        self.0.iter().find_map(|token| match token {
            StylePropertyToken::Percentage(val)
            | StylePropertyToken::Dimension(val)
            | StylePropertyToken::Number(val) => Some(Some(val.into())),
            StylePropertyToken::Identifier(ident) => match ident.as_str() {
                "none" => Some(None),
                _ => None,
            },
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Option<UiRect>`].
    ///
    /// Optional values are handled by this function, so if only one value is present it is used as `top`, `right`, `bottom` and `left`,
    /// otherwise values are applied in the following order: `top`, `right`, `bottom` and `left`.
    ///
    /// Note that it is not possible to create a [`UiRect`] with only `top` value, since it'll be understood to replicated it on all fields.
    pub fn rect(&self) -> Result<UiRect, ElementsError> {
        match self.0.len() {
            1 => self.0[0].val().map(UiRect::all),
            2 => {
                let top_bottom = self.0[0].val()?;
                let left_right = self.0[1].val()?;
                Ok(UiRect::new(left_right, left_right, top_bottom, top_bottom))
            }
            3 => {
                let top = self.0[0].val()?;
                let left_right = self.0[1].val()?;
                let bottom = self.0[2].val()?;
                Ok(UiRect::new(left_right, left_right, top, bottom))
            }
            4 => {
                let top = self.0[0].val()?;
                let right = self.0[1].val()?;
                let bottom = self.0[2].val()?;
                let left = self.0[3].val()?;
                Ok(UiRect::new(left, right, top, bottom))
            }
            _ => Err(ElementsError::InvalidPropertyValue(format!(
                "Can't extract rect from `{}`",
                self.to_string()
            ))),
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = "".to_string();
        for value in self.0.iter() {
            result.push_str(&value.to_string());
        }
        result
    }
}

fn parse_style_propery_value(value: &str) -> Result<StyleProperty, ElementsError> {
    let mut input = cssparser::ParserInput::new(value);
    let mut parser = cssparser::Parser::new(&mut input);
    let mut values: SmallVec<[StylePropertyToken; 8]> = SmallVec::new();
    loop {
        let next = parser.next();
        match next {
            Ok(token) => values.push(token.clone().try_into().map_err(|e| {
                ElementsError::InvalidPropertyValue(format!(
                    "Can't parse `{}` (invalid token `{:?}`: {:?}",
                    value, token, e
                ))
            })?),
            Err(e) if e.kind == BasicParseErrorKind::EndOfInput => break,
            Err(e) => {
                return Err(ElementsError::InvalidPropertyValue(format!(
                    "Can't parse `{}`: {:?}",
                    value, e
                )))
            }
        }
    }
    Ok(StyleProperty(values))
}

impl TryFrom<&str> for StyleProperty {
    type Error = ElementsError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_style_propery_value(value)
    }
}

impl TryFrom<String> for StyleProperty {
    type Error = ElementsError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        parse_style_propery_value(&value)
    }
}

impl TryFrom<&String> for StyleProperty {
    type Error = ElementsError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        parse_style_propery_value(value.as_str())
    }
}
