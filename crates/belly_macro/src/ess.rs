use std::fmt::Debug;

use bevy::prelude::Deref;
use proc_macro2::{Delimiter, Span, TokenTree};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, Token};

macro_rules! throw {
    ($span:expr, $msg:literal $($args:tt)*) => {
        return Err(syn::Error::new($span, format!($msg $($args)*)))
    };
}

pub struct EssDefinition {
    pub ident: syn::Ident,
    pub stylesheet: StyleSheet,
}

impl syn::parse::Parse for EssDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let stylesheet = input.parse()?;
        Ok(EssDefinition { ident, stylesheet })
    }
}

pub trait HasSpace {
    fn has_space(&self, other: &Self) -> bool;
}

impl HasSpace for Span {
    // checks if there a whitespace between two adjacent spans
    // when span locations available (when parsing from strings)
    // use cheap comparations, fallback to cheaty span debug
    // representation otherwise
    fn has_space(&self, other: &Self) -> bool {
        fn start(repr: &str) -> &str {
            repr.split("(")
                .nth(1)
                .expect("[span] start not splited by (")
                .split("..")
                .next()
                .expect("[span] start not splited by ..")
        }
        fn end(repr: &str) -> &str {
            repr.split("(")
                .nth(1)
                .expect("[span] end not splited by (")
                .split("..")
                .nth(1)
                .expect("[span] end not splited by ..")
                .strip_suffix(")")
                .expect("[span] end ) suffix not stripped")
        }
        let (el, ec, sl, sc) = (
            self.end().line,
            self.end().column,
            other.start().line,
            other.start().column,
        );
        if (el, ec, sl, sc) != (0, 0, 0, 0) {
            return el != sl || ec != sc;
        } else {
            end(format!("{self:?}").as_str()) != start(format!("{other:?}").as_str())
        }
    }
}

pub fn parse_docs(input: &mut syn::parse::ParseStream) -> syn::Result<Vec<String>> {
    let mut docs = vec![];
    while input.peek(Token![#]) {
        input.parse::<Token![#]>()?;
        let content;
        bracketed!(content in input);
        let doc_ident = content.parse::<syn::Ident>()?;
        if doc_ident.to_string().as_str() != "doc" {
            throw!(doc_ident.span(), "Expected `doc` ident")
        }
        content.parse::<syn::Token![=]>()?;
        let doc = content.parse::<syn::LitStr>()?;
        docs.push(doc.value());
    }
    Ok(docs)
}

pub fn parse_ident(input: syn::parse::ParseStream) -> syn::Result<String> {
    let span = input.span();
    let ident = input.step(|cursor| {
        let mut rest = *cursor;
        let mut value = format!("");
        let mut last = cursor.span();
        let mut first_iter = true;
        while let Some((tt, next)) = rest.token_tree() {
            let span = tt.span();
            let has_space = last.has_space(&span);
            // println!("tttry {tt:?}, space: {has_space}, last: {last:?}, start: {:?}", tt.span().start());
            last = span.clone();
            if !first_iter && has_space {
                return Ok((value, rest));
            }
            first_iter = false;
            match tt {
                TokenTree::Ident(i) => value = value + i.to_string().as_str(),
                TokenTree::Punct(p) if p.as_char() == '-' => value = value + "-",
                _ if !value.is_empty() => return Ok((value, rest)),
                e => throw!(span, "Unexpected token: {e:?}"),
            }
            rest = next;
        }
        if value.is_empty() {
            throw!(span, "Expected ';'")
        } else {
            Ok((value, rest))
        }
    })?;
    if ident.is_empty() {
        throw!(span, "Expected ident")
    }
    Ok(ident)
}

#[derive(Debug, Clone)]
pub enum SelectorToken {
    Tag(String),
    Id(String),
    Class(String),
    State(String),
    AnyChild,
    DirectChild,
    Any,
}

impl SelectorToken {
    pub fn add_str(&mut self, s: &str, span: &Span) -> Result<(), syn::Error> {
        let value = match self {
            Self::Id(i) | Self::Tag(i) | Self::Class(i) | Self::State(i) => i.clone() + s,
            e => throw!(span.clone(), "Trying to add ident to {e:?}"),
        };
        *self = match self {
            Self::Id(_) => Self::Id(value),
            Self::Tag(_) => Self::Tag(value),
            Self::Class(_) => Self::Class(value),
            Self::State(_) => Self::State(value),
            e => throw!(span.clone(), "Trying to add ident to {e:?}"),
        };
        Ok(())
    }
}

impl std::fmt::Display for SelectorToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tag(s) => write!(f, "{s}"),
            Self::Id(s) => write!(f, "#{s}"),
            Self::Class(s) => write!(f, ".{s}"),
            Self::State(s) => write!(f, ":{s}"),
            Self::AnyChild => write!(f, " "),
            Self::DirectChild => write!(f, " > "),
            Self::Any => write!(f, "*"),
        }
    }
}

#[derive(Deref, Debug)]
pub struct Selector(Vec<SelectorToken>);

impl Selector {
    pub fn push(&mut self, token: SelectorToken) {
        match (token, self.last()) {
            (SelectorToken::DirectChild, None)
            | (SelectorToken::AnyChild, None)
            | (SelectorToken::AnyChild, Some(SelectorToken::AnyChild))
            | (SelectorToken::AnyChild, Some(SelectorToken::DirectChild)) => {}
            (SelectorToken::DirectChild, Some(SelectorToken::AnyChild)) => {
                *self.0.last_mut().unwrap() = SelectorToken::DirectChild
            }
            (token, _) => self.0.push(token),
        }
    }
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            self.0
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join("")
                .as_str(),
        )
    }
}

#[derive(Debug)]
pub enum StyleValueToken {
    Dimension(f32, String),
    Percent(f32),
    Num(f32),
    Ident(String),
    String(String),
    Color(String),
    Values(Vec<StyleValueToken>),
    Function(String, Vec<StyleValueToken>),
    Comma,
    Slash,
}

impl StyleValueToken {
    fn to_string(&self) -> String {
        match self {
            Self::Ident(i) => format!("{i}"),
            Self::Dimension(b, s) => format!("{b}{s}"),
            Self::Percent(p) => format! {"{p}%"},
            Self::Num(n) => format! {"{n}"},
            Self::String(s) => format!("\"{s}\""),
            Self::Color(s) => format!("#{s}"),
            Self::Values(v) => format!(
                "{}",
                v.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            Self::Function(name, args) => format!(
                "{}({})",
                name,
                args.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Self::Comma => format!(", "),
            Self::Slash => format!("/"),
        }
    }
}

#[derive(Deref, Debug)]
pub struct StyleValue(Vec<StyleValueToken>);

impl std::fmt::Display for StyleValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = self
            .0
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ")
            .replace(" , ", ",");
        f.write_str(result.as_str())
    }
}

impl syn::parse::Parse for StyleValue {
    fn parse(mut input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut value = vec![];
        loop {
            // it is possible when parsing function arguments
            if input.is_empty() {
                return Ok(StyleValue(value));
            }
            let span = input.span();
            if input.peek(Token![;]) {
                return Ok(StyleValue(value));
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                value.push(StyleValueToken::Comma);
            } else if input.peek(Token![/]) {
                input.parse::<Token![/]>()?;
                value.push(StyleValueToken::Slash);
            } else if input.peek(Token![#]) {
                input.parse::<Token![#]>()?;
                let color = input.step(|cursor| {
                    let mut rest = *cursor;
                    let mut color = format!("");
                    let mut last = cursor.span();
                    let mut first_iter = true;
                    while let Some((tt, next)) = rest.token_tree() {
                        let span = tt.span();
                        let has_space = last.has_space(&span);
                        // println!("space: {has_space}, first: {first_iter}, tt: {tt:?}");
                        last = span.clone();
                        if !first_iter && has_space {
                            return Ok((color, rest));
                        }
                        first_iter = false;
                        match tt {
                            TokenTree::Ident(i) => color = color + i.to_string().as_str(),
                            TokenTree::Literal(l) => color = color + l.to_string().as_str(),
                            TokenTree::Punct(p) if p.as_char() == ';' => return Ok((color, rest)),
                            e => throw!(span, "Unsupported token {e:?}"),
                        }
                        rest = next;
                    }
                    return Ok((color, rest));
                })?;
                const VALID_COLOR_CHARS: &'static [char] = &[
                    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
                    'A', 'B', 'C', 'D', 'E', 'F',
                ];
                if let Some(invalid) = color
                    .chars()
                    .filter(|c| !VALID_COLOR_CHARS.contains(c))
                    .next()
                {
                    throw!(span, "Invalid color code for character '{invalid}'")
                }
                match color.len() {
                    3 | 4 | 6 | 8 => {}
                    _ => throw!(span, "Invalid color value"),
                }
                value.push(StyleValueToken::Color(color));
            } else if input.peek(syn::Lit) {
                let lit = input.parse::<syn::Lit>()?;
                if let syn::Lit::Str(s) = lit {
                    value.push(StyleValueToken::String(s.value()))
                } else {
                    let mut base = None;
                    let mut suffix = None;
                    let mut percent = false;
                    if let syn::Lit::Float(f) = lit {
                        base = Some(f.base10_parse::<f32>()?);
                        if !f.suffix().is_empty() {
                            suffix = Some(f.suffix().to_string());
                        }
                    } else if let syn::Lit::Int(i) = lit {
                        base = Some(i.base10_parse::<f32>()?);
                        if !i.suffix().is_empty() {
                            suffix = Some(i.suffix().to_string());
                        }
                    }
                    if input.peek(Token![%]) {
                        input.parse::<Token![%]>()?;
                        percent = true;
                    }
                    if base.is_none() {
                        throw!(span, "Invalid literal token")
                    }
                    if percent {
                        value.push(StyleValueToken::Percent(base.unwrap()))
                    } else if suffix.is_some() {
                        value.push(StyleValueToken::Dimension(base.unwrap(), suffix.unwrap()))
                    } else {
                        value.push(StyleValueToken::Num(base.unwrap()))
                    }
                }
            } else {
                let ident = parse_ident(&mut input)?;
                if !input.peek(syn::token::Paren) {
                    // just an ident
                    value.push(StyleValueToken::Ident(ident))
                } else {
                    // function
                    let content;
                    syn::parenthesized!(content in input);
                    let args: Punctuated<StyleValue, Token![,]> =
                        content.parse_terminated(StyleValue::parse)?;
                    value.push(StyleValueToken::Function(
                        ident,
                        args.into_iter()
                            .map(|mut value| {
                                if value.len() == 1 {
                                    value.0.pop().unwrap()
                                } else {
                                    StyleValueToken::Values(value.0)
                                }
                            })
                            .collect(),
                    ));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct StyleProperty {
    comments: Vec<String>,
    name: String,
    value: StyleValue,
}

impl std::fmt::Display for StyleProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "  {}: {};\n", self.name, self.value.to_string())
        } else {
            write!(f, "{}: {};", self.name, self.value.to_string())
        }
    }
}

impl syn::parse::Parse for StyleProperty {
    fn parse(mut input: syn::parse::ParseStream) -> syn::Result<Self> {
        let comments = parse_docs(&mut input)?;
        let name = parse_ident(&mut input)?;
        // println!("name: {name:?}, iiinput: {input:?}");
        input.parse::<Token![:]>()?;
        let value = input.parse::<StyleValue>()?;
        Ok(StyleProperty {
            comments,
            name,
            value,
        })
    }
}

#[derive(Debug)]
pub struct StyleRule {
    comments: Vec<String>,
    selector: Selector,
    properties: Vec<StyleProperty>,
}

impl std::fmt::Display for StyleRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            for comment in self.comments.iter() {
                write!(f, "/**{comment}*/\n")?;
            }
            write!(f, "{} {{\n", self.selector)?;

            for (idx, property) in self.properties.iter().enumerate() {
                for (pidx, comment) in property.comments.iter().enumerate() {
                    let nl = if idx == 0 || pidx != 0 { "" } else { "\n" };
                    write!(f, "{nl}  /**{comment}*/\n")?;
                }
                write!(f, "  {}: {};\n", property.name, property.value)?;
            }
            write!(f, "}}\n")
        } else {
            write!(
                f,
                "{} {{ {} }}",
                self.selector.to_string(),
                self.properties
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        }
    }
}

impl syn::parse::Parse for StyleRule {
    fn parse(mut input: syn::parse::ParseStream) -> syn::Result<Self> {
        let comments = parse_docs(&mut input)?;
        // parse selector
        let selector = input.step(|cursor| {
            let mut rest = *cursor;
            let mut selector = Selector(vec![]);
            let mut last = cursor.span();
            let mut token = None;
            while let Some((tt, next)) = rest.token_tree() {
                let span = tt.span();
                let has_space = last.has_space(&span);
                last = span.clone();
                // println!("{tt:?}, space: {has_space}, last pos:{:?}", last);
                match tt {
                    TokenTree::Ident(i) => {
                        if token.is_none() {
                            if has_space {
                                selector.push(SelectorToken::AnyChild);
                            }
                            token = Some(SelectorToken::Tag(i.to_string()));
                        } else {
                            if has_space {
                                selector.push(token.clone().unwrap());
                                selector.push(SelectorToken::AnyChild);
                                token = Some(SelectorToken::Tag(i.to_string()));
                            } else {
                                token
                                    .as_mut()
                                    .unwrap()
                                    .add_str(i.to_string().as_str(), &span)?;
                            }
                        }
                    }
                    TokenTree::Punct(p) if p.as_char() == '-' => {
                        if token.is_none() {
                            if has_space {
                                selector.push(SelectorToken::AnyChild);
                            }
                            token = Some(SelectorToken::Tag("-".to_string()));
                        } else {
                            if has_space {
                                selector.push(token.unwrap());
                                selector.push(SelectorToken::AnyChild);
                                token = Some(SelectorToken::Tag("-".to_string()));
                            } else {
                                token.as_mut().unwrap().add_str("-", &span)?;
                            }
                        }
                    }
                    TokenTree::Punct(p) if p.as_char() == '.' => {
                        if token.is_some() {
                            selector.push(token.unwrap())
                        }
                        if has_space {
                            selector.push(SelectorToken::AnyChild);
                        }
                        token = Some(SelectorToken::Class("".to_string()));
                    }
                    TokenTree::Punct(p) if p.as_char() == ':' => {
                        if token.is_some() {
                            selector.push(token.unwrap())
                        }
                        if has_space {
                            selector.push(SelectorToken::AnyChild);
                        }
                        token = Some(SelectorToken::State("".to_string()));
                    }
                    TokenTree::Punct(p) if p.as_char() == '#' => {
                        if token.is_some() {
                            selector.push(token.unwrap())
                        }
                        if has_space {
                            selector.push(SelectorToken::AnyChild);
                        }
                        token = Some(SelectorToken::Id("".to_string()));
                    }
                    TokenTree::Punct(p) if p.as_char() == '*' => {
                        // let source  = span.unwrap().source_text().expect("need source");
                        //throw!(span, "Source: '{}'", source);
                        // span.eq
                        // throw!(span, "has space: {has_space}");
                        if token.is_some() {
                            selector.push(token.unwrap())
                        }
                        if has_space {
                            selector.push(SelectorToken::AnyChild);
                        }
                        selector.push(SelectorToken::Any);
                        token = None;
                    }
                    TokenTree::Punct(p) if p.as_char() == '>' => {
                        if token.is_some() {
                            selector.push(token.unwrap())
                        }
                        selector.push(SelectorToken::DirectChild);
                        token = None;
                    }
                    TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                        if token.is_some() {
                            selector.push(token.unwrap());
                        }
                        return Ok((selector, rest));
                    }
                    e => throw!(span, "Unsupported selector: {e:?}"),
                }
                rest = next;
            }
            if token.is_some() {
                selector.push(token.unwrap());
            }
            Ok((selector, rest))
        })?;

        let mut properties = vec![];
        // parse properties
        if input.is_empty() {
            throw!(input.span(), "Expected style properties block");
        } else {
            let content;
            braced!(content in input);
            let props: Punctuated<StyleProperty, Token![;]> =
                content.parse_terminated(StyleProperty::parse)?;
            for prop in props {
                properties.push(prop);
            }
            // properties = props.iter().cloned().collect();
            // props.
            // println!("rest input: {props:?}");
        }

        Ok(StyleRule {
            comments,
            selector,
            properties,
        })
    }
}

#[derive(Debug, Default)]
pub struct StyleSheet(Vec<StyleRule>);

impl std::fmt::Display for StyleSheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            for rule in self.0.iter() {
                write!(f, "{rule:#}\n")?;
            }
        } else {
            let s = self
                .0
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            f.write_str(s.as_str())?;
        }
        Ok(())
    }
}

impl syn::parse::Parse for StyleSheet {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut result = vec![];
        while !input.is_empty() {
            result.push(input.parse()?);
        }
        Ok(StyleSheet(result))
    }
}

#[cfg(test)]
mod test {

    use proc_macro2::TokenStream;

    use super::*;

    #[test]
    fn test_selectors() {
        let selecors = &[
            "body {  }",
            "body.class {  }",
            "body > * {  }",
            ".button-foreground > * {  }",
            "tag#id {  }",
            "tag:state {  }",
            "tag child {  }",
            "tag > direct-child {  }",
            "tag.class > direct-child:state {  }",
            ":state any-child.class {  }",
            "tag-name.class-name :some-state #cool-id {  }",
        ];
        for src in selecors {
            let stream: TokenStream = src.parse().unwrap();
            let rule: StyleRule = syn::parse2(stream).unwrap();
            println!("Selector: {:?}", rule.selector);
            assert_eq!(rule.to_string().as_str(), *src);
        }
    }
    #[test]
    fn test_rules() {
        let rules = &[
            // single
            r#"body { prop: value; }"#,
            r#"body { some-prop: value; }"#,
            r#"body { some-bigger-prop: value; }"#,
            r#"body { prop: value; prop2: value2; }"#,
            r#"body { prop: 25px; }"#,
            r#"body { prop: 25%; }"#,
            r#"body { prop: "25%"; }"#,
            // multi-value
            r#"body { padding: 2px 10%; }"#,
            // multi-prop
            r#"body { padding: 2px 10%; margin-left: 10px; }"#,
            // multi-prop with delim
            r#"body { stylebox: "hello.png", 2px 10%, 10%; margin-left: 10px; }"#,
            // simple func
            r#"body { grid-template-rows: flex(1); }"#,
            // complex func
            r#"body { grid-template-rows: repeat(2, flex(1)); }"#,
            r#"body { grid-template-columns: min-content flex(1); }"#,
            // test slash formating
            r#"body { grid-row: 2 / span 2; }"#,
        ];
        for src in rules {
            println!("Checking '{src}'");
            let stream: TokenStream = src.parse().unwrap();
            let rule: StyleRule = syn::parse2(stream).unwrap();
            println!("Rule: {:?}", rule);
            assert_eq!(rule.to_string().as_str(), *src);
        }
    }

    #[test]
    fn test_multiline() {
        let src = r#"
            body { 
                stylebox: "hello.png", 2px 10%, 10%;
                margin-left: 10px; 
            }
        "#;
        let target = r#"body { stylebox: "hello.png", 2px 10%, 10%; margin-left: 10px; }"#;
        println!("Checking '{src}'");
        let stream: TokenStream = src.parse().unwrap();
        let rule: StyleRule = syn::parse2(stream).unwrap();
        println!("Rule: {:?}", rule);
        assert_eq!(rule.to_string().as_str(), target);
    }

    #[test]
    fn test_stylesheet() {
        let src = r#"body { prop: value; }
div { some-prop: value; }
span { some-bigger-prop: value; }"#;
        println!("Checking '{src}'");
        let stream: TokenStream = src.parse().unwrap();
        let stylesheet: StyleSheet = syn::parse2(stream).unwrap();
        println!("StyleSheet: {:?}", stylesheet);
        assert_eq!(stylesheet.to_string().as_str(), src);
    }

    #[test]
    fn test_colors() {
        let valid = &[
            r#"body { prop: #123fde; }"#,
            r#"body { prop: #ef32ab; }"#,
            r#"body { prop: #EF32AE; }"#,
            r#"body { prop: #ABCDEF; }"#,
            r#"body { prop: #123456; }"#,
            r#"body { prop: #123456 white; }"#,
            r#"body { prop: #ABCDEF red; }"#,
        ];
        let invalid = &[r#"body { prop: #xx2dab; }"#, r#"body { prop: #113exo; }"#];
        for src in valid {
            println!("Checking '{src}'");
            let stream: TokenStream = src.parse().unwrap();
            let rule: StyleRule = syn::parse2(stream).unwrap();
            println!("Rule: {:?}", rule);
            assert_eq!(rule.to_string().as_str(), *src);
        }
        for src in invalid {
            println!("Checking '{src}'");
            let stream: TokenStream = src.parse().unwrap();
            let rule: Result<StyleRule, syn::Error> = syn::parse2(stream);
            assert_eq!(rule.is_err(), true);
        }
    }

    #[test]
    fn test_pretty_comments() {
        let src = r#"
empty-rule {
}

/** comment body */
body {
  prop: value;
}

/** comment div */
div {
  some-prop: value;

  /** comment second-prop */
  second-prop: value;
}

.button-foreground * {
  /** comment first-prop */
  first-prop: value;

  /**  ***********************
   **  comment second-prop
   **  with mutliline comment
   **  ***********************/
  second-prop: value;

  /** another way */
  /** to pass */
  /** multiline comments */
  third-prop: value;
}
"#;
        println!("Checking '{src}'");
        let stream: TokenStream = src.parse().unwrap();
        let stylesheet: StyleSheet = syn::parse2(stream).unwrap();
        println!("StyleSheet: {:?}", stylesheet);
        assert_eq!(format!("{stylesheet:#}").as_str().trim(), src.trim());
    }
}
