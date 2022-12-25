use bevy::prelude::{error, warn};
use smallvec::{smallvec, SmallVec};

use cssparser::*;
use tagstr::{AsTag, Tag};

use crate::{
    ess::selector::{Selector, SelectorElement},
    ess::StyleRule,
    property::StyleProperty,
    ElementsError, PropertyExtractor, PropertyTransformer, Variant,
};

pub struct StyleSheetParser {
    transformer: PropertyTransformer,
    extractor: PropertyExtractor,
}

impl StyleSheetParser {
    pub fn new(transformer: PropertyTransformer, extractor: PropertyExtractor) -> StyleSheetParser {
        StyleSheetParser {
            extractor,
            transformer,
        }
    }
    pub fn parse(&self, content: &str) -> SmallVec<[StyleRule; 8]> {
        let mut input = ParserInput::new(content);
        let mut parser = Parser::new(&mut input);
        RuleListParser::new_for_stylesheet(&mut parser, self)
            .into_iter()
            .filter_map(|result| match result {
                Ok(rule) => Some(rule),
                Err((err, rule)) => {
                    error!(
                        "Failed to parse rule: {}. Error: {}",
                        rule,
                        format_error(err)
                    );
                    None
                }
            })
            .collect()
    }
}

fn format_error(error: ParseError<ElementsError>) -> String {
    let error_description = match error.kind {
        cssparser::ParseErrorKind::Basic(b) => match b {
            cssparser::BasicParseErrorKind::UnexpectedToken(token) => {
                format!("Unexpected token {}", token.to_css_string())
            }
            cssparser::BasicParseErrorKind::EndOfInput => "End of input".to_string(),
            cssparser::BasicParseErrorKind::AtRuleInvalid(token) => {
                format!("At rule isn't supported {}", token)
            }
            cssparser::BasicParseErrorKind::AtRuleBodyInvalid => {
                "At rule isn't supported".to_string()
            }
            cssparser::BasicParseErrorKind::QualifiedRuleInvalid => "Invalid rule".to_string(),
        },
        cssparser::ParseErrorKind::Custom(c) => c.to_string(),
    };

    format!(
        "{} at {}:{}",
        error_description, error.location.line, error.location.column
    )
}

#[derive(Default)]
enum NextElement {
    #[default]
    Tag,
    Class,
    Attribute,
}

impl<'i> QualifiedRuleParser<'i> for &StyleSheetParser {
    type Prelude = Selector;
    type QualifiedRule = StyleRule;
    type Error = ElementsError;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let mut elements: SmallVec<[SelectorElement; 8]> = smallvec![];

        let mut next = NextElement::Tag;

        while let Ok(token) = input.next_including_whitespace() {
            use cssparser::Token::*;
            match token {
                Ident(v) => {
                    match next {
                        NextElement::Tag => {
                            elements.insert(0, SelectorElement::Tag(v.to_string().as_tag()))
                        }
                        NextElement::Class => {
                            elements.insert(0, SelectorElement::Class(v.to_string().as_tag()))
                        }
                        NextElement::Attribute => {
                            elements.insert(0, SelectorElement::State(v.to_string().as_tag()))
                        }
                    };
                    next = NextElement::Tag;
                }
                IDHash(v) => {
                    if v.is_empty() {
                        return Err(input.new_custom_error(ElementsError::InvalidSelector));
                    } else {
                        elements.insert(0, SelectorElement::Id(v.to_string().as_tag()));
                    }
                }
                WhiteSpace(_) => elements.insert(0, SelectorElement::AnyChild),
                Delim(c) if *c == '.' => next = NextElement::Class,
                Delim(c) if *c == '*' => elements.insert(0, SelectorElement::Any),
                Colon => next = NextElement::Attribute,
                _ => {
                    warn!("Unexpected token: {:?}", token);
                    let token = token.to_css_string();
                    return Err(input.new_custom_error(ElementsError::UnexpectedToken(token)));
                }
            }
        }

        if elements.is_empty() {
            return Err(input.new_custom_error(ElementsError::InvalidSelector));
        }

        // Remove noise the trailing white spaces, if any
        while !elements.is_empty() {
            if elements.last().unwrap().is_any_child() {
                elements.pop();
            } else if elements.first().unwrap().is_any_child() {
                elements.remove(0);
            } else {
                break;
            }
        }

        Ok(Selector::new(elements))
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &cssparser::ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let mut rule = StyleRule {
            selector: prelude,
            properties: Default::default(),
        };

        for property in DeclarationListParser::new(input, PropertyParser) {
            match property {
                Ok((name, property)) => {
                    if self.extractor.is_compound_property(name) {
                        let extracted = match self.extractor.extract(name, Variant::style(property))
                        {
                            Err(e) => return Err(input.new_custom_error(e)),
                            Ok(extracted) => extracted,
                        };
                        for (name, property) in extracted {
                            rule.properties.insert(name, property);
                        }
                    } else {
                        match self.transformer.transform(name, Variant::style(property)) {
                            Ok(variant) => {
                                rule.properties.insert(name, variant);
                            }
                            Err(e) => return Err(input.new_custom_error(e)),
                        }
                    }
                }
                Err((err, a)) => println!("Failed: {:?} ({})", err, a),
            }
        }

        Ok(rule)
    }
}

impl<'i> AtRuleParser<'i> for &StyleSheetParser {
    type Prelude = ();
    type AtRule = StyleRule;
    type Error = ElementsError;
}

struct PropertyParser;

impl<'i> DeclarationParser<'i> for PropertyParser {
    type Declaration = (Tag, StyleProperty);

    type Error = ElementsError;

    fn parse_value<'t>(
        &mut self,
        name: cssparser::CowRcStr<'i>,
        parser: &mut Parser<'i, 't>,
    ) -> Result<Self::Declaration, ParseError<'i, ElementsError>> {
        let mut tokens = smallvec![];
        for token in parse_values(parser)? {
            match token.try_into() {
                Ok(t) => tokens.push(t),
                Err(_) => continue,
            }
        }

        Ok((name.to_string().as_tag(), StyleProperty(tokens)))
    }
}

impl<'i> AtRuleParser<'i> for PropertyParser {
    type Prelude = ();
    type AtRule = (Tag, StyleProperty);
    type Error = ElementsError;
}

fn parse_values<'i, 'tt>(
    parser: &mut Parser<'i, 'tt>,
) -> Result<SmallVec<[Token<'i>; 8]>, ParseError<'i, ElementsError>> {
    let mut values = SmallVec::new();

    while let Ok(token) = parser.next_including_whitespace() {
        values.push(token.clone())
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use crate::{
        property::{PropertyValue, StylePropertyToken},
        ExtractProperty, TransformProperty,
    };

    use super::*;
    use bevy::utils::HashMap;

    fn transform(v: Variant) -> Result<PropertyValue, ElementsError> {
        match v {
            Variant::Style(s) => Ok(PropertyValue::new(s)),
            _ => Err(ElementsError::InvalidPropertyValue(format!(
                "Smth wrong with tests"
            ))),
        }
    }

    struct TestParser {
        extractor: PropertyExtractor,
        transformer: PropertyTransformer,
    }

    impl TestParser {
        fn new() -> TestParser {
            let mut transformers: HashMap<Tag, TransformProperty> = Default::default();
            for tag in "a b c d e f g h i j k l".split(" ") {
                transformers.insert(tag.as_tag(), transform);
                transformers.insert(format!("{}-{}", tag, tag).as_tag(), transform);
            }
            let mut extractors: HashMap<Tag, ExtractProperty> = Default::default();
            extractors.insert("compound".as_tag(), |_| {
                let mut map = HashMap::default();
                map.insert(
                    "a".as_tag(),
                    PropertyValue::new(StyleProperty(smallvec![StylePropertyToken::Identifier(
                        "a".to_string()
                    )])),
                );
                map.insert(
                    "b".as_tag(),
                    PropertyValue::new(StyleProperty(smallvec![StylePropertyToken::Identifier(
                        "b".to_string()
                    )])),
                );
                Ok(map)
            });

            let validator = PropertyTransformer::new(transformers);
            let extractor = PropertyExtractor::new(extractors);
            TestParser {
                transformer: validator,
                extractor,
            }
        }

        fn parse(&self, content: &str) -> SmallVec<[StyleRule; 8]> {
            let parser = StyleSheetParser::new(self.transformer.clone(), self.extractor.clone());
            parser.parse(content)
            // StyleSheetParser::parse(content, self.validator.clone(), self.extractor.clone())
        }

        // fn
    }

    #[test]
    fn parse_empty() {
        let parser = TestParser::new();
        assert!(
            parser.parse("").is_empty(),
            "Should return an empty list of rules"
        );
        assert!(
            parser.parse("{}").is_empty(),
            "\"{{}}\" Should return an empty list of rules"
        );
        assert!(
            parser.parse(" {}").is_empty(),
            "\" {{}}\" Should return an empty list of rules"
        );
        assert!(
            parser.parse("# {}").is_empty(),
            "\"# {{}}\" Should return an empty list of rules"
        );
        assert!(
            parser.parse("@@@ {}").is_empty(),
            "Should return an empty list of rules"
        );
        assert!(
            parser.parse("{}{}").is_empty(),
            "Should return an empty list of rules"
        );
    }

    #[test]
    fn parse_single_name_selector_no_property() {
        let rules = TestParser::new().parse("#id {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.entries();
        assert_eq!(tree.len(), 1, "Should have a single selector node");

        let node = &tree[0];
        assert_eq!(node.len(), 1, "Should have a single selector");
        assert!(node.has_id("id".as_tag()), "Should have a id selector");

        assert!(rule.properties.is_empty(), "Should have no token");
    }

    #[test]
    fn parse_single_class_selector_no_property() {
        let rules = TestParser::new().parse(".class {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.entries();
        assert_eq!(tree.len(), 1, "Should have a single selector node");

        let node = &tree[0];
        assert_eq!(node.len(), 1, "Should have a single selector");
        assert!(
            node.has_class("class".as_tag()),
            "Should have a class selector"
        );

        assert!(rule.properties.is_empty(), "Should have no token");
    }

    #[test]
    fn parse_single_component_selector_no_property() {
        let rules = TestParser::new().parse("button {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.entries();
        assert_eq!(tree.len(), 1, "Should have a single selector node");

        let node = &tree[0];
        assert_eq!(node.len(), 1, "Should have a single selector");
        assert!(
            node.has_tag("button".as_tag()),
            "Should have a tag selector"
        );

        assert!(rule.properties.is_empty(), "Should have no token");
    }

    #[test]
    fn parse_single_complex_class_selector_no_property() {
        let rules = TestParser::new().parse(".a.b.c.d.e.f.g {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.entries();
        assert_eq!(tree.len(), 1, "Should have a single selector node");

        let node = &tree[0];
        assert_eq!(node.len(), 7, "Should have a 7 selector class");
        for cls in "a b c d e f g".split(" ") {
            assert!(node.has_class(cls.as_tag()), "Should have {} class", cls);
        }

        assert!(rule.properties.is_empty(), "Should have no token");
    }

    #[test]
    fn parse_single_composed_selector_no_property() {
        let rules = TestParser::new().parse("a.b#c.d {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.entries();
        assert_eq!(tree.len(), 1, "Should have a single selector node");

        let node = &tree[0];
        assert_eq!(node.len(), 4, "Should have a 4 selectors");
        assert!(node.has_tag("a".as_tag()), "Should have 'a' tag");
        assert!(node.has_class("b".as_tag()), "Should have 'b' class");
        assert!(node.has_class("d".as_tag()), "Should have 'd' class");
        assert!(node.has_id("c".as_tag()), "Should have 'c' id");

        assert!(rule.properties.is_empty(), "Should have no token");
    }

    #[test]
    fn parse_multiple_composed_selector_no_property() {
        let rules = TestParser::new().parse("a.b #c .d e#f .g.h i j.k#l {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.entries();
        assert_eq!(tree.len(), 7, "Should have a single selector node");
        assert!(tree[0].has_tag("a".as_tag()), "Should has 'a' tag");
        assert!(tree[0].has_class("b".as_tag()), "Should has 'b' class");
        assert!(tree[1].has_id("c".as_tag()), "Should has 'c' id");
        assert!(tree[2].has_class("d".as_tag()), "Should has 'd' class");
        assert!(tree[3].has_tag("e".as_tag()), "Should has 'd' tag");
        assert!(tree[3].has_id("f".as_tag()), "Should has 'f' id");
        assert!(tree[4].has_class("g".as_tag()), "Should has 'g' class");
        assert!(tree[4].has_class("h".as_tag()), "Should has 'h' class");
        assert!(tree[5].has_tag("i".as_tag()), "Should has 'i' tag");
        assert!(tree[6].has_tag("j".as_tag()), "Should has 'j' tag");
        assert!(tree[6].has_class("k".as_tag()), "Should has 'k' class");
        assert!(tree[6].has_id("l".as_tag()), "Should has 'l' id");

        assert!(rule.properties.is_empty(), "Should have no properties");
    }

    #[test]
    fn parse_single_token() {
        let rules = TestParser::new().parse("a {b: c}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let properties = &rules[0].properties;

        assert_eq!(properties.len(), 1, "Should have a single property");
        assert!(
            properties.contains_key(&"b".as_tag()),
            "Should have a property named \"b\""
        );

        let values = properties
            .get(&"b".as_tag())
            .unwrap()
            .downcast_ref::<StyleProperty>()
            .unwrap();

        assert_eq!(values.len(), 1, "Should have a single property value");

        match &values[0] {
            StylePropertyToken::Identifier(ident) => assert_eq!(ident, "c"),
            _ => panic!("Should have a property value of type identifier token"),
        }
    }

    #[test]
    fn parse_multiple_complex_properties() {
        let rules = TestParser::new().parse(
            r#"a {
            a: a;
            b: 0px;
            c: #f; 
            d: h i j; 
            e-e: 100%;
            f: 15.3px 3%;
            i: 12.9;
            j: "str";
            k: p q #r #s "t" 1 45.67% 33px;
        }"#,
        );

        assert_eq!(rules.len(), 1, "Should have a single rule");

        let properties = &rules[0].properties;

        use StylePropertyToken::*;
        let expected = [
            ("a", vec![Identifier("a".to_string())]),
            ("b", vec![Dimension(0.0.into())]),
            ("c", vec![Hash("f".to_string())]),
            (
                "d",
                vec![
                    Identifier("h".to_string()),
                    Identifier("i".to_string()),
                    Identifier("j".to_string()),
                ],
            ),
            ("e-e", vec![Percentage(100.0.into())]),
            ("f", vec![Dimension(15.3.into()), Percentage(3.0.into())]),
            ("i", vec![Number(12.9.into())]),
            ("j", vec![String("str".to_string())]),
            (
                "k",
                vec![
                    Identifier("p".to_string()),
                    Identifier("q".to_string()),
                    Hash("r".to_string()),
                    Hash("s".to_string()),
                    String("t".to_string()),
                    Number(1.0.into()),
                    Percentage(45.67.into()),
                    Dimension(33.0.into()),
                ],
            ),
        ];

        assert_eq!(properties.len(), expected.len(), "{:?}", properties);
        expected.into_iter().for_each(|(name, values)| {
            assert!(properties.contains_key(&name.as_tag()));
            values
                .iter()
                .zip(
                    properties
                        .get(&name.as_tag())
                        .unwrap()
                        .downcast_ref::<StyleProperty>()
                        .unwrap()
                        .iter(),
                )
                .for_each(|(expected, token)| {
                    assert_eq!(token, expected);
                })
        });
    }

    #[test]
    fn parse_multiple_rules() {
        let rules = TestParser::new().parse(r#"a{a:a}a{a:a}a{a:a}a{a:a}"#);

        assert_eq!(rules.len(), 4, "Should have 4 rules");

        for rule in rules {
            assert_eq!(
                rule.selector.tail().len(),
                1,
                "Should have only a single component"
            );
            assert!(
                rule.selector.tail().has_tag("a".as_tag()),
                "Should has 'a' tag"
            );

            match rule
                .properties
                .get(&"a".as_tag())
                .expect("Should have a single property named \"a\"")
                .downcast_ref::<StyleProperty>()
                .unwrap()
                .iter()
                .next()
                .expect("Should have a single property value")
            {
                StylePropertyToken::Identifier(a) => assert_eq!(a, "a"),
                _ => panic!("Should have only a single property value of type identifier"),
            }
        }
    }

    #[test]
    fn parse_compound_properties() {
        let rules = TestParser::new().parse("a { compound: valid }");
        assert_eq!(rules.len(), 1, "Should have a two rules (a and b)");
        let rule = &rules[0];
        assert_eq!(
            rule.selector.entries().len(),
            1,
            "Rule should have single selector"
        );
        assert!(
            rule.selector.tail().has_tag("a".as_tag()),
            "Selector should have single property"
        );

        let properties = &rule.properties;

        use StylePropertyToken::*;
        let expected = [
            ("a", vec![Identifier("a".to_string())]),
            ("b", vec![Identifier("b".to_string())]),
        ];
        assert_eq!(properties.len(), expected.len(), "{:?}", properties);
        expected.into_iter().for_each(|(name, values)| {
            assert!(properties.contains_key(&name.as_tag()));
            values
                .iter()
                .zip(
                    properties
                        .get(&name.as_tag())
                        .unwrap()
                        .downcast_ref::<StyleProperty>()
                        .unwrap()
                        .iter(),
                )
                .for_each(|(expected, token)| {
                    assert_eq!(token, expected);
                })
        });
    }
}
