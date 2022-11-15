use bevy::{prelude::default, utils::HashSet};
use smallvec::SmallVec;
use tagstr::Tag;


pub (crate) struct StyleRule {

}

#[derive(Default)]
struct SelectorIndex(Option<usize>);

pub (crate) enum SelectorElement {
    AnyChild,
    Id(Tag),
    Class(Tag),
    Tag(Tag),
    Attribute(Tag)
}

impl SelectorElement {
    pub fn is_any_child(&self) -> bool {
        match self {
            SelectorElement::AnyChild => true,
            _ => false
        }
    }

    pub fn is_value(&self) -> bool {
        !self.is_any_child()
    }

    pub fn describes_node(&self, node: &impl EmlNode) -> bool {
        match self {
            SelectorElement::Id(id) => node.id() == Some(*id),
            SelectorElement::Attribute(attr) => node.has_attribute(attr),
            SelectorElement::Tag(tag) => node.tag() == *tag,
            SelectorElement::Class(class) => node.has_class(class),
            _ => false
        }
    }
}

type SelectorElements = SmallVec<[SelectorElement; 8]>;

pub struct SelectorSlice<'a> {
    offset: usize,
    elements: &'a SelectorElements
}

impl<'a> SelectorSlice<'a> {
    fn new(elements: &'a SelectorElements) -> SelectorSlice<'a> {
        SelectorSlice { elements, offset: 0 }
    }
    fn next(&self) -> Option<SelectorSlice<'a>> {
        let mut offset = self.offset;
        let elements = self.elements;
        if elements[offset].is_any_child() {
            offset += 1;
            if offset >= elements.len() {
                return None;
            } else {
                return Some(SelectorSlice { offset, elements });
            }
        }
        
        while offset < elements.len() && !elements[offset].is_any_child() {
            offset += 1;
        }

        if offset >= elements.len() {
            return None;
        } else {
            return Some(SelectorSlice { offset, elements });
        }
    }

    pub fn is_any_child(&self) -> bool {
        self.elements[self.offset].is_any_child()
    }

    pub fn is_value(&self) -> bool {
        !self.is_any_child()
    }

    pub fn describes_node(&self, node: &impl EmlNode) -> bool {
        let mut offset = self.offset;
        let elements = self.elements;
        if elements[offset].is_any_child() {
            return false;
        }
        while offset < elements.len() && elements[offset].is_value() {
            if elements[offset].describes_node(node) {
                offset += 1
            } else {
                return false;
            }
        }
        true
    }

}

#[derive(Default)]
pub (crate) struct Selector {
    index: SelectorIndex,
    elements: SelectorElements
}


impl Selector {
    pub fn new(mut elements: SelectorElements) -> Selector {
        Selector { elements, ..default() }
    }

    pub fn slice(&self) -> SelectorSlice {
        SelectorSlice { offset: 0, elements: &self.elements }
    }

    pub fn matches(&self, branch: impl EmlBranch) -> bool {
        let slice = SelectorSlice::new(&self.elements);
        branch.root().fits(&slice)
    }
}

pub trait EmlBranch {
    type Node: EmlNode;
    fn root(&self) -> Self::Node;
}

pub trait EmlNode: Sized {
    fn id(&self) -> Option<Tag>;
    fn tag(&self) -> Tag;
    fn has_attribute(&self, tag: &Tag) -> bool;
    fn has_class(&self, class: &Tag) -> bool;

    fn next(&self) -> Option<Self>;

    fn fits(&self, selector: &SelectorSlice) -> bool {
        if selector.is_any_child() {
            let next_selector = selector.next().unwrap();
            if self.fits(&next_selector) {
                return true;
            }
            if let Some(next_node) = self.next() {
                next_node.fits(&next_selector) || next_node.fits(selector)
            } else {
                false
            }
        } else if selector.describes_node(self) {
            match (self.next(), selector.next()) {
                (None, None) => true,
                (Some(next_node), Some(next_slice)) => next_node.fits(&next_slice),
                (Some(next_node), None) => next_node.fits(selector),
                (None, Some(_slice)) => false
            }
        } else if let Some(next) = self.next() {
            next.fits(selector)
        } else {
            false
        }
    }
}

struct TestBranch(Vec<TestNodeData>);

impl<'a> EmlBranch for &'a TestBranch {
    type Node = TestNode<'a>;

    fn root(&self) -> Self::Node {
        TestNode { 
            index: 0,
            branch: self
        }
    }
   
}

#[derive(Default)]
struct TestNodeData {
    id: Option<Tag>,
    tag: Tag,
    classes: HashSet<Tag>,
    attributes: HashSet<Tag>
}

struct TestNode<'a> {
    index: usize,
    branch: &'a TestBranch
}

impl<'a> EmlNode for TestNode<'a> {
    fn id(&self) -> Option<Tag> {
        self.branch.0[self.index].id
    }
    fn tag(&self) -> Tag {
        self.branch.0[self.index].tag
    }
    fn has_attribute(&self, tag: &Tag) -> bool {
        self.branch.0[self.index].attributes.contains(tag)
    }
    fn has_class(&self, class: &Tag) -> bool {
        self.branch.0[self.index].classes.contains(class)
    }
    fn next(&self) -> Option<Self> {
        let index = self.index + 1;
        if index >= self.branch.0.len() {
            None
        } else {
            Some(TestNode { index, branch: self.branch })
        }
    }
}

impl From<&str> for Selector {
    fn from(source: &str) -> Self {
        use cssparser::{Parser, ParserInput, ToCss, Token::*};
        use tagstr::*;
        const NEXT_TAG: u8 = 0;
        const NEXT_CLASS: u8 = 1;
        const NEXT_ATTR: u8 = 2;
        let mut selector = Selector::default();
        // selector.elements.push(SelectorElement::AnyChild);
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);
        let mut next = NEXT_TAG;
        while let Ok(token) = parser.next_including_whitespace() {
            match token {
                Ident(v) => { 
                    match next {
                        NEXT_TAG => selector.elements.push(SelectorElement::Tag(v.to_string().as_tag())),
                        NEXT_CLASS => selector.elements.push(SelectorElement::Class(v.to_string().as_tag())),
                        NEXT_ATTR => selector.elements.push(SelectorElement::Attribute(v.to_string().as_tag())),
                        _ => panic!("Invalid NEXT_TAG")
                    };
                    next = NEXT_TAG;
                },
                IDHash(v) => {
                    if v.is_empty() {
                        panic!("Invalid #id selector");
                    } else {
                        selector.elements.push(SelectorElement::Id(v.to_string().as_tag()));
                    }
                }
                WhiteSpace(_) => selector.elements.push(SelectorElement::AnyChild),
                Colon => next = NEXT_ATTR,
                Delim(c) if *c == '.' => next = NEXT_CLASS,
                _ => panic!("Unexpected token: {}", token.to_css_string())
            }
        }

        selector
    }
}

impl From<Selector> for TestBranch {
    fn from(selector: Selector) -> Self {
        let mut branch = TestBranch(vec!());
        let mut node = TestNodeData::default();
        let mut has_values = false;
        let void = |_| ();
        for element in selector.elements {
            match element {
                SelectorElement::AnyChild => {
                    if has_values {
                        branch.0.push(node);
                        node = TestNodeData::default();
                    }
                    has_values = false;
                    continue;
                    
                },
                SelectorElement::Attribute(attr) => void(node.attributes.insert(attr)),
                SelectorElement::Class(class) => void(node.classes.insert(class)),
                SelectorElement::Id(id) => node.id = Some(id),
                SelectorElement::Tag(tag) => node.tag = tag
            };
            has_values = true;
        }
        if has_values {
            branch.0.push(node);
        }
        branch
    }
}

impl From<&str> for TestBranch {
    fn from(selector: &str) -> Self {
        let selector: Selector = selector.into();
        selector.into()
    }
}

#[cfg(test)]
mod test {
    use super::TestBranch;
    use super::*;
    use tagstr::*;

    #[test]
    fn selector_construct_test_branch() {
        // single element
        let branch: TestBranch = "div".into();
        assert_eq!(branch.0.len(), 1);

        // spaces
        let branch: TestBranch = "div ".into();
        assert_eq!(branch.0.len(), 1);
        let branch: TestBranch = " div ".into();
        assert_eq!(branch.0.len(), 1);

        // attribute
        let branch: TestBranch = " div:attr ".into();
        assert_eq!(branch.0.len(), 1);
        assert!(branch.0[0].attributes.contains(&"attr".as_tag()));

        // class
        let branch: TestBranch = " div.cls ".into();
        assert_eq!(branch.0.len(), 1);
        assert!(branch.0[0].classes.contains(&"cls".as_tag()));

        // id
        let branch: TestBranch = " div#id ".into();
        assert_eq!(branch.0.len(), 1);
        assert_eq!(branch.0[0].id, Some("id".as_tag()));

        // complex
        let branch: TestBranch = " div#id.cls span:attr ".into();
        assert_eq!(branch.0.len(), 2);
        assert_eq!(branch.0[0].tag, "div".as_tag());
        assert_eq!(branch.0[1].tag, "span".as_tag());
        assert_eq!(branch.0[0].id, Some("id".as_tag()));
        assert_eq!(branch.0[0].classes.contains(&"cls".as_tag()), true);
        assert_eq!(branch.0[1].attributes.contains(&"attr".as_tag()), true);
    }

    #[test]
    fn selector_single_element() {
        let branch: TestBranch = "div".into();
        let valid_selector: Selector = "div".into();
        let invalid_selector: Selector = "span".into();
        assert!(valid_selector.matches(&branch));
        assert!(!invalid_selector.matches(&branch));

        let branch: TestBranch = "div.cls".into();
        let valid_selector: Selector = ".cls".into();
        let invalid_selector: Selector = ":span".into();
        assert!(valid_selector.matches(&branch));
        assert!(!invalid_selector.matches(&branch));
    }

    #[test]
    fn selector_multi_elements() {
        let branch: TestBranch = "div.red#id:pressed span.green span.red".into();
        let valid_selectors: &[&str] = &[
            "span",
            "div span",
            ".red",
            ".green .red",
            "#id:pressed .red",
            "div span span",
            ".red .red"
        ];
        for src in valid_selectors {
            let selector: Selector = src.clone().into();
            assert!(selector.matches(&branch), "Selector '{}' should be matched", src);
        }
        let invalid_selectors: &[&str] = &[
            "#id",
            "#id .green",
            "span div",
            "div .green",
            ".red .green",
            ":pressed #id",
            ".red div",
            "#id div",
            "#id.red .red .green",
            "div span span .red",
            ".red .green :pressed"
        ];
        for src in invalid_selectors {
            let selector: Selector = src.clone().into();
            assert!(!selector.matches(&branch), "Selector '{}' shouldn't be matched", src);
        }
    }
}