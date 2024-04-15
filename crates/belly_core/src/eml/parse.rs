use super::asset::{EmlElement, EmlLoader, EmlNode};
use super::Variant;
use crate::{ess::StyleProperty, ElementsError};
use std::fmt::Display;
use tagstr::{AsTag, Tag};

const NS_STYLE: &str = "s";

pub(crate) fn parse(source: &str, loader: &EmlLoader) -> Result<EmlNode, ParseError> {
    let source = EmlSource::new(source);
    parse_internal(&source, loader).map_err(|e| ParseError::new(e, &source))
}

enum Error {
    InvalidElement(String, roxmltree::TextPos),
    InvalidStyleValue(String, roxmltree::TextPos),
    InvalidDocumentStructure(String, roxmltree::TextPos),
    Internal(roxmltree::Error),
}

impl Error {
    fn pos(&self) -> roxmltree::TextPos {
        match self {
            Error::InvalidElement(_, pos) => *pos,
            Error::InvalidDocumentStructure(_, pos) => *pos,
            Error::InvalidStyleValue(_, pos) => *pos,
            Error::Internal(e) => e.pos(),
        }
    }
}

#[derive(Debug)]
pub struct ParseError(String);

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ParseError {}

impl ParseError {
    fn new(err: Error, source: &EmlSource) -> ParseError {
        let msg = match &err {
            Error::Internal(e) => format!("{}", e),
            Error::InvalidElement(e, pos) => format!("Invalid element: {} at {}", e, pos),
            Error::InvalidDocumentStructure(msg, pos) => {
                format!("Invalid document structure: {} at {}", msg, pos)
            }
            Error::InvalidStyleValue(msg, pos) => format!("{} at {}", msg, pos),
        };

        let pos = err.pos();
        let posmsg = format!(" at {}:{}", pos.row, pos.col);
        let msg = format!(
            "{} at {}:{}",
            msg.replace(&posmsg, ""),
            pos.row - source.line_offset,
            pos.col
        );
        let msglen = msg.chars().count();
        let lineidx = if pos.row > 0 { pos.row - 1 } else { pos.row } as usize;
        let line = source.data.lines().nth(lineidx).unwrap();
        let linelen = line.chars().count();
        let pos = pos.col as usize;
        let suffix0len = if linelen > msglen {
            linelen - msglen + 1
        } else {
            1
        };
        let suffix1len = if linelen > msglen {
            2
        } else {
            msglen - linelen + 2
        };
        let suffix2len = linelen.max(msglen) - pos + 1;
        let empty = "";
        let errmsg = format!(
            "{msg} {empty:-<s0$}.\n{line}{empty: <s1$}|\n{empty: <pos$}^{empty:-<s2$}`\n",
            s0 = suffix0len,
            s1 = suffix1len,
            s2 = suffix2len,
            pos = pos
        );
        ParseError(errmsg)
    }
}

struct EmlSource {
    line_offset: u32,
    data: String,
}

impl EmlSource {
    fn new(data: &str) -> EmlSource {
        let prefix = format!("<skip:root xmlns:skip=\"skip\" xmlns:{NS_STYLE}=\"{NS_STYLE}\">\n");
        let suffix = "\n</skip:root>";
        let line_offset = 1;
        let data = prefix + data + suffix;
        EmlSource { line_offset, data }
    }
}

fn parse_internal(source: &EmlSource, loader: &EmlLoader) -> Result<EmlNode, Error> {
    let document = roxmltree::Document::parse(&source.data);
    match document {
        Err(e) => Err(Error::Internal(e)),
        Ok(doc) => parse_root(doc.root(), loader),
    }
}

fn parse_root(node: roxmltree::Node, loader: &EmlLoader) -> Result<EmlNode, Error> {
    let ns = node.tag_name().namespace();
    let doc = node.document();
    let pos = doc.text_pos_at(node.position());
    if node.is_root() || ns == Some("skip") {
        let children: Vec<_> = node.children().filter(|n| n.is_element()).collect();
        if children.len() != 1 {
            return Err(Error::InvalidDocumentStructure(
                "Node should has exactly one child".to_string(),
                pos,
            ));
        }
        parse_root(children[0], loader)
    } else {
        walk(node, loader)
    }
}

fn walk(node: roxmltree::Node, loader: &EmlLoader) -> Result<EmlNode, Error> {
    let doc = node.document();
    let pos = doc.text_pos_at(node.position());
    if node.is_text() {
        let text = node.text().unwrap();
        let text = text.trim();
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        Ok(EmlNode::Text(text))
    } else if node.is_element() && node.tag_name().name() == "slot" {
        let slot_name = node.attribute("replace").ok_or_else(|| {
            Error::InvalidElement(format!("<slot> tag should have 'for' attribute."), pos)
        })?;
        let mut slot_elements: Vec<EmlNode> = vec![];
        for ch in node.children() {
            slot_elements.push(walk(ch, loader)?);
        }
        Ok(EmlNode::Slot(slot_name.as_tag(), slot_elements))
    } else if node.is_element() {
        let node_name = node.tag_name().name().as_tag();
        if !loader.registry.has(node_name) {
            return Err(Error::InvalidElement(node_name.to_string(), pos));
        }

        let mut elem = EmlElement::new(node_name);
        for attr in node.attributes() {
            let pos = doc.text_pos_at(attr.position());
            let name = if let Some(ns) = attr.namespace() {
                if ns == NS_STYLE {
                    validate_style(attr.name().as_tag(), attr.value(), loader).map_err(|e| {
                        Error::InvalidStyleValue(
                            format!(
                                "Invalid value for {NS_STYLE}:{} attribute: {}",
                                attr.name(),
                                e
                            ),
                            pos,
                        )
                    })?;
                }
                format!("{}:{}", ns, attr.name())
            } else {
                attr.name().to_string()
            };
            elem.params.insert(name, attr.value().to_string());
        }
        for ch in node.children() {
            if ch.is_text() {
                let text = ch.text().unwrap();
                let text = text.trim();
                let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
                if text.len() > 0 {
                    elem.children.push(EmlNode::Text(text));
                }
            } else if ch.is_element() {
                elem.children.push(walk(ch, loader)?);
            }
        }
        Ok(EmlNode::Element(elem))
    } else {
        Err(Error::InvalidDocumentStructure(
            format!("Invalid element: {node:?}"),
            pos,
        ))
    }
}

fn validate_style(name: Tag, value: &str, loader: &EmlLoader) -> Result<(), ElementsError> {
    let props = Variant::style(TryInto::<StyleProperty>::try_into(value)?);
    if loader.extractor.is_compound_property(name) {
        loader.extractor.extract(name, props)?;
    } else {
        loader.transformer.transform(name, props)?;
    }
    Ok(())
}
