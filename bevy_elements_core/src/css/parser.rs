// use smallvec::{SmallVec, smallvec};
// use bevy::prelude::error;

// use cssparser::*;
// use tagstr::{Tag, tag, AsTag};

// use crate::{
//     ElementsError,
//     css::style::{StyleRule, Selector, SelectorElement }
// };

// pub(crate) struct StyleSheetParser;

// impl StyleSheetParser {
//     pub(crate) fn parse(content: &str) -> SmallVec<[StyleRule; 8]> {
//         let mut input = ParserInput::new(content);
//         let mut parser = Parser::new(&mut input);

//         RuleListParser::new_for_stylesheet(&mut parser, StyleSheetParser)
//             .into_iter()
//             .filter_map(|result| match result {
//                 Ok(rule) => Some(rule),
//                 Err((err, rule)) => {
//                     error!(
//                         "Failed to parse rule: {}. Error: {}",
//                         rule,
//                         format_error(err)
//                     );
//                     None
//                 }
//             })
//             .collect()
//     }
// }

// fn format_error(error: ParseError<ElementsError>) -> String {
//     let error_description = match error.kind {
//         cssparser::ParseErrorKind::Basic(b) => match b {
//             cssparser::BasicParseErrorKind::UnexpectedToken(token) => {
//                 format!("Unexpected token {}", token.to_css_string())
//             }
//             cssparser::BasicParseErrorKind::EndOfInput => "End of input".to_string(),
//             cssparser::BasicParseErrorKind::AtRuleInvalid(token) => {
//                 format!("At rule isn't supported {}", token)
//             }
//             cssparser::BasicParseErrorKind::AtRuleBodyInvalid => {
//                 "At rule isn't supported".to_string()
//             }
//             cssparser::BasicParseErrorKind::QualifiedRuleInvalid => "Invalid rule".to_string(),
//         },
//         cssparser::ParseErrorKind::Custom(c) => c.to_string(),
//     };

//     format!(
//         "{} at {}:{}",
//         error_description, error.location.line, error.location.column
//     )
// }


// #[derive(Default)]
// enum NextElement {
//     #[default]
//     Tag,
//     Class,
//     Attribute
// }

// impl<'i> QualifiedRuleParser<'i> for StyleSheetParser {
//     type Prelude = Selector;
//     type QualifiedRule = StyleRule;
//     type Error = ElementsError;

//     fn parse_prelude<'t>(
//         &mut self,
//         input: &mut Parser<'i, 't>,
//     ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
//         let mut elements = smallvec![];

//         let mut next = NextElement::Tag;

//         while let Ok(token) = input.next_including_whitespace() {
//             use cssparser::Token::*;
//             match token {
//                 Ident(v) => { 
//                     match next {
//                         NextElement::Tag => elements.push(SelectorElement::Tag(v.to_string().as_tag())),
//                         NextElement::Class => elements.push(SelectorElement::Class(v.to_string().as_tag())),
//                         NextElement::Attribute => elements.push(SelectorElement::Attribute(v.to_string().as_tag()))
//                     };
//                     next = NextElement::Tag;
//                 },
//                 IDHash(v) => {
//                     if v.is_empty() {
//                         return Err(input.new_custom_error(ElementsError::InvalidSelector));
//                     } else {
//                         elements.push(SelectorElement::Id(v.to_string().as_tag()));
//                     }
//                 }
//                 WhiteSpace(_) => elements.push(SelectorElement::AnyChild),
//                 Delim(c) if *c == '.' => next = NextElement::Class,
//                 Delim(c) if *c == ':' => next = NextElement::Attribute,
//                 _ => {
//                     let token = token.to_css_string();
//                     return Err(input.new_custom_error(ElementsError::UnexpectedToken(token)));
//                 }
//             }
//         }

//         if elements.is_empty() {
//             return Err(input.new_custom_error(ElementsError::InvalidSelector));
//         }

//         // Remove noise the trailing white spaces, if any
//         while !elements.is_empty() && elements.last().unwrap() == &SelectorElement::AnyChild {
//             elements.remove(elements.len() - 1);
//         }

//         Ok(Selector::new(elements))
//     }

//     fn parse_block<'t>(
//         &mut self,
//         prelude: Self::Prelude,
//         _start: &cssparser::ParserState,
//         input: &mut Parser<'i, 't>,
//     ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
//         let mut rule = StyleRule {
//             selector: prelude,
//             properties: Default::default(),
//         };

//         for property in DeclarationListParser::new(input, PropertyParser) {
//             match property {
//                 Ok((name, property)) => {
//                     rule.properties.insert(name, property);
//                 }
//                 Err((err, a)) => println!("Failed: {:?} ({})", err, a),
//             }
//         }

//         Ok(rule)
//     }
// }

// impl<'i> AtRuleParser<'i> for StyleSheetParser {
//     type Prelude = ();
//     type AtRule = StyleRule;
//     type Error = EcssError;
// }

// struct PropertyParser;

// impl<'i> DeclarationParser<'i> for PropertyParser {
//     type Declaration = (String, PropertyValues);

//     type Error = EcssError;

//     fn parse_value<'t>(
//         &mut self,
//         name: cssparser::CowRcStr<'i>,
//         parser: &mut Parser<'i, 't>,
//     ) -> Result<Self::Declaration, ParseError<'i, EcssError>> {
//         let mut tokens = smallvec![];
//         for token in parse_values(parser)? {
//             match token.try_into() {
//                 Ok(t) => tokens.push(t),
//                 Err(_) => continue,
//             }
//         }

//         Ok((name.to_string(), PropertyValues(tokens)))
//     }
// }

// impl<'i> AtRuleParser<'i> for PropertyParser {
//     type Prelude = ();
//     type AtRule = (String, PropertyValues);
//     type Error = EcssError;
// }

// fn parse_values<'i, 'tt>(
//     parser: &mut Parser<'i, 'tt>,
// ) -> Result<SmallVec<[Token<'i>; 8]>, ParseError<'i, EcssError>> {
//     let mut values = SmallVec::new();

//     while let Ok(token) = parser.next_including_whitespace() {
//         values.push(token.clone())
//     }

//     Ok(values)
// }

// #[cfg(test)]
// mod tests {
//     use crate::property::PropertyToken;

//     use super::*;

//     #[test]
//     fn parse_empty() {
//         assert!(
//             StyleSheetParser::parse("").is_empty(),
//             "Should return an empty list of rules"
//         );
//         assert!(
//             StyleSheetParser::parse("{}").is_empty(),
//             "\"{{}}\" Should return an empty list of rules"
//         );
//         assert!(
//             StyleSheetParser::parse(" {}").is_empty(),
//             "\" {{}}\" Should return an empty list of rules"
//         );
//         assert!(
//             StyleSheetParser::parse("# {}").is_empty(),
//             "\"# {{}}\" Should return an empty list of rules"
//         );
//         assert!(
//             StyleSheetParser::parse("@@@ {}").is_empty(),
//             "Should return an empty list of rules"
//         );
//         assert!(
//             StyleSheetParser::parse("{}{}").is_empty(),
//             "Should return an empty list of rules"
//         );
//     }

//     #[test]
//     fn parse_single_name_selector_no_property() {
//         let rules = StyleSheetParser::parse("#id {}");
//         assert_eq!(rules.len(), 1, "Should have a single rule");

//         let rule = &rules[0];
//         let tree = rule.selector.get_parent_tree();
//         assert_eq!(tree.len(), 1, "Should have a single selector node");

//         let node = &tree[0];
//         assert_eq!(tree.len(), 1, "Should have a single selector");

//         match node[0] {
//             SelectorElement::Name(name) => assert_eq!(name, "id"),
//             _ => panic!("Should have a name selector"),
//         }

//         assert!(rule.properties.is_empty(), "Should have no token");
//     }

//     #[test]
//     fn parse_single_class_selector_no_property() {
//         let rules = StyleSheetParser::parse(".class {}");
//         assert_eq!(rules.len(), 1, "Should have a single rule");

//         let rule = &rules[0];
//         let tree = rule.selector.get_parent_tree();
//         assert_eq!(tree.len(), 1, "Should have a single selector node");

//         let node = &tree[0];
//         assert_eq!(tree.len(), 1, "Should have a single selector");

//         match node[0] {
//             SelectorElement::Class(name) => assert_eq!(name, "class"),
//             _ => panic!("Should have a class selector"),
//         }

//         assert!(rule.properties.is_empty(), "Should have no token");
//     }

//     #[test]
//     fn parse_single_component_selector_no_property() {
//         let rules = StyleSheetParser::parse("button {}");
//         assert_eq!(rules.len(), 1, "Should have a single rule");

//         let rule = &rules[0];
//         let tree = rule.selector.get_parent_tree();
//         assert_eq!(tree.len(), 1, "Should have a single selector node");

//         let node = &tree[0];
//         assert_eq!(tree.len(), 1, "Should have a single selector");

//         match node[0] {
//             SelectorElement::Component(name) => assert_eq!(name, "button"),
//             _ => panic!("Should have a class selector"),
//         }

//         assert!(rule.properties.is_empty(), "Should have no token");
//     }

//     #[test]
//     fn parse_single_complex_class_selector_no_property() {
//         let rules = StyleSheetParser::parse(".a.b.c.d.e.f.g {}");
//         assert_eq!(rules.len(), 1, "Should have a single rule");

//         let rule = &rules[0];
//         let tree = rule.selector.get_parent_tree();
//         assert_eq!(tree.len(), 1, "Should have a single selector node");

//         let node = &tree[0];
//         assert_eq!(node.len(), 7, "Should have a 7 selector class");

//         use SelectorElement::*;
//         let expected: SmallVec<[SelectorElement; 8]> = smallvec![
//             Class("a".to_string()),
//             Class("b".to_string()),
//             Class("c".to_string()),
//             Class("d".to_string()),
//             Class("e".to_string()),
//             Class("f".to_string()),
//             Class("g".to_string()),
//         ];

//         expected
//             .into_iter()
//             .zip(node.into_iter())
//             .for_each(|(expected, element)| {
//                 assert_eq!(expected, **element);
//             });

//         assert!(rule.properties.is_empty(), "Should have no token");
//     }

//     #[test]
//     fn parse_single_composed_selector_no_property() {
//         let rules = StyleSheetParser::parse("a.b#c.d {}");
//         assert_eq!(rules.len(), 1, "Should have a single rule");

//         let rule = &rules[0];
//         let tree = rule.selector.get_parent_tree();
//         assert_eq!(tree.len(), 1, "Should have a single selector node");

//         let node = &tree[0];
//         assert_eq!(node.len(), 4, "Should have a 4 selectors");

//         use SelectorElement::*;
//         let expected: SmallVec<[SelectorElement; 8]> = smallvec![
//             Component("a".to_string()),
//             Class("b".to_string()),
//             Name("c".to_string()),
//             Class("d".to_string()),
//         ];

//         expected
//             .into_iter()
//             .zip(node.into_iter())
//             .for_each(|(expected, element)| {
//                 assert_eq!(expected, **element);
//             });

//         assert!(rule.properties.is_empty(), "Should have no token");
//     }

//     #[test]
//     fn parse_multiple_composed_selector_no_property() {
//         let rules = StyleSheetParser::parse("a.b #c .d e#f .g.h i j.k#l {}");
//         assert_eq!(rules.len(), 1, "Should have a single rule");

//         let rule = &rules[0];
//         let tree = rule.selector.get_parent_tree();
//         assert_eq!(tree.len(), 7, "Should have a single selector node");

//         use SelectorElement::*;
//         let expected: SmallVec<[SmallVec<[SelectorElement; 8]>; 8]> = smallvec![
//             smallvec![Component("a".to_string()), Class("b".to_string())],
//             smallvec![Name("c".to_string())],
//             smallvec![Class("d".to_string())],
//             smallvec![Component("e".to_string()), Name("f".to_string())],
//             smallvec![Class("g".to_string()), Class("h".to_string())],
//             smallvec![Component("i".to_string())],
//             smallvec![
//                 Component("j".to_string()),
//                 Class("k".to_string()),
//                 Name("l".to_string())
//             ],
//         ];

//         expected
//             .into_iter()
//             .zip(tree.into_iter())
//             .for_each(|(node_expected, node)| {
//                 node_expected
//                     .into_iter()
//                     .zip(node)
//                     .for_each(|(expected, element)| {
//                         assert_eq!(expected, *element);
//                     });
//             });

//         assert!(rule.properties.is_empty(), "Should have no properties");
//     }

//     #[test]
//     fn parse_single_token() {
//         let rules = StyleSheetParser::parse("a {b: c}");
//         assert_eq!(rules.len(), 1, "Should have a single rule");

//         let properties = &rules[0].properties;

//         assert_eq!(properties.len(), 1, "Should have a single property");
//         assert!(
//             properties.contains_key(&"b".to_string()),
//             "Should have a property named \"b\""
//         );

//         let values = properties.get(&"b".to_string()).unwrap();

//         assert_eq!(values.len(), 1, "Should have a single property value");

//         match &values[0] {
//             PropertyToken::Identifier(ident) => assert_eq!(ident, "c"),
//             _ => panic!("Should have a property value of type identifier token"),
//         }
//     }

//     #[test]
//     fn parse_multiple_complex_properties() {
//         let rules = StyleSheetParser::parse(
//             r#"a {
//             b: c;
//             d: 0px;
//             e: #f; 
//             g: h i j; 
//             k-k: 100%;
//             l: 15.3px 3%;
//             m: 12.9;
//             n: "str";
//             o: p q #r #s "t" 1 45.67% 33px;
//         }"#,
//         );

//         assert_eq!(rules.len(), 1, "Should have a single rule");

//         let properties = &rules[0].properties;

//         use PropertyToken::*;
//         let expected = [
//             ("b", vec![Identifier("c".to_string())]),
//             ("d", vec![Dimension(0.0)]),
//             ("e", vec![Hash("f".to_string())]),
//             (
//                 "g",
//                 vec![
//                     Identifier("h".to_string()),
//                     Identifier("i".to_string()),
//                     Identifier("j".to_string()),
//                 ],
//             ),
//             ("k-k", vec![Percentage(100.0)]),
//             ("l", vec![Dimension(15.3), Percentage(3.0)]),
//             ("m", vec![Number(12.9)]),
//             ("n", vec![String("str".to_string())]),
//             (
//                 "o",
//                 vec![
//                     Identifier("p".to_string()),
//                     Identifier("q".to_string()),
//                     Hash("r".to_string()),
//                     Hash("s".to_string()),
//                     String("t".to_string()),
//                     Number(1.0),
//                     Percentage(45.67),
//                     Dimension(33.0),
//                 ],
//             ),
//         ];

//         assert_eq!(properties.len(), expected.len(), "{:?}", properties);
//         expected.into_iter().for_each(|(name, values)| {
//             assert!(properties.contains_key(name));
//             values
//                 .iter()
//                 .zip(properties.get(name).unwrap().iter())
//                 .for_each(|(expected, token)| {
//                     assert_eq!(token, expected);
//                 })
//         });
//     }

//     #[test]
//     fn parse_multiple_rules() {
//         let rules = StyleSheetParser::parse(r#"a{a:a}a{a:a}a{a:a}a{a:a}"#);

//         assert_eq!(rules.len(), 4, "Should have 4 rules");

//         for rule in rules {
//             match rule.selector.get_parent_tree()[0][0] {
//                 SelectorElement::Component(a) => assert_eq!(a, "a"),
//                 _ => panic!("Should have only a single component \"a\""),
//             }

//             match rule
//                 .properties
//                 .get(&"a".to_string())
//                 .expect("Should have a single property named \"a\"")
//                 .iter()
//                 .next()
//                 .expect("Should have a single property value")
//             {
//                 PropertyToken::Identifier(a) => assert_eq!(a, "a"),
//                 _ => panic!("Should have only a single property value of type identifier"),
//             }
//         }
//     }
// }