use std::fmt::Display;

use roxmltree;
use tagstr::AsTag;

use crate::property::PropertyValues;

use super::{EmlElement, EmlLoader, EmlNode};

const NS_STYLE: &str = "s";

pub(crate) fn parse(source: &str, loader: &EmlLoader) -> Result<EmlElement, ParseError> {
    let source = EmlSource::new(source);
    parse_internal(&source, loader).map_err(|e| ParseError::new(e, &source))
}

enum Error {
    InvalidElement(String, roxmltree::TextPos),
    InvalidStyleValue(String, roxmltree::TextPos),
    InvalidDocumentStructure(String, roxmltree::TextPos),
    ValidationError(String, roxmltree::TextPos),
    Internal(roxmltree::Error),
}

impl Error {
    fn pos(&self) -> roxmltree::TextPos {
        match self {
            Error::InvalidElement(_, pos) => *pos,
            Error::InvalidDocumentStructure(_, pos) => *pos,
            Error::InvalidStyleValue(_, pos) => *pos,
            Error::ValidationError(_, pos) => *pos,
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
            Error::ValidationError(msg, pos) => format!("{} at {}", msg, pos),
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

fn parse_internal(source: &EmlSource, loader: &EmlLoader) -> Result<EmlElement, Error> {
    let document = roxmltree::Document::parse(&source.data);
    match document {
        Err(e) => Err(Error::Internal(e)),
        Ok(doc) => walk(doc.root(), loader),
    }
}

fn walk(node: roxmltree::Node, loader: &EmlLoader) -> Result<EmlElement, Error> {
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
        return walk(children[0], loader);
    }
    if !node.is_element() {
        return Err(Error::InvalidDocumentStructure(
            "Non-element node found".to_string(),
            pos,
        ));
    }
    let node_name = node.tag_name().name().as_tag();
    if !loader.registry.has_builder(node_name) {
        return Err(Error::InvalidElement(node_name.to_string(), pos));
    }

    let mut elem = EmlElement::new(node_name);
    for attr in node.attributes() {
        let pos = doc.text_pos_at(attr.position());
        let name = if let Some(ns) = attr.namespace() {
            if ns == NS_STYLE {
                let props = TryInto::<PropertyValues>::try_into(attr.value()).map_err(|e| {
                    Error::InvalidStyleValue(
                        format!(
                            "Invalid value for {NS_STYLE}:{} attribute: {}",
                            attr.name(),
                            e
                        ),
                        pos,
                    )
                })?;
                loader
                    .validator
                    .validate(attr.name().as_tag(), &props)
                    .map_err(|e| {
                        Error::ValidationError(
                            format!(
                                "Error validating value for {NS_STYLE}:{} attribute: {}",
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
        elem.attributes.insert(name, attr.value().to_string());
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
            elem.children.push(EmlNode::Element(walk(ch, loader)?));
        }
    }
    Ok(elem)
}
