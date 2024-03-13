// usage from inside belly crate:
// cargo run -p belly_cli -- gen widget-reference > docs/widgets.md
use std::{collections::HashMap, fs::File, io::BufReader};

use clap::{Parser, Subcommand};
use rustdoc_json;
use rustdoc_types::{Crate, Id, Item, ItemEnum, ItemKind, Module, Type};
use serde_json::from_reader;

#[derive(Debug, Parser)]
#[command(name = "cargo-polako")]
#[command(about = "Polako CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(subcommand)]
    Gen(Gen),
}

#[derive(Debug, Subcommand)]
#[command(args_conflicts_with_subcommands = true)]
enum Gen {
    StyleReference,
    WidgetReference,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Command::Gen(Gen::StyleReference) => gen_style_docs(),
        Command::Gen(Gen::WidgetReference) => gen_widget_docs(),
    }
}

fn gen_widget_docs() {
    let json_path = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("crates/belly_widgets/Cargo.toml")
        // .manifest_path("crates/belly_core/Cargo.toml")
        .build()
        .unwrap();

    let f = File::open(&json_path)
        .unwrap_or_else(|_| panic!("Could not open {}", json_path.to_str().unwrap()));
    let rdr = BufReader::new(f);
    let crt: Crate = from_reader(rdr).unwrap_or_else(|e| panic!("Can't parse json: {e:?}"));
    let mut widgets = fetch_widgets(&crt);
    widgets.sort_by_key(|k| k.name.clone());
    for widget in widgets.iter() {
        println!("## {}", widget.name);
        if let Some(extends) = &widget.extends {
            println!("\nextends: `<{}>`\n", extends.name);
        }
        let body = widget.docs_body();
        if !body.is_empty() {
            println!("\n{body}\n")
        }
        let params = widget.docs_params();
        if !params.is_empty() {
            println!("\nParams:\n\n{params}")
        }
        println!("");
    }
}

fn gen_style_docs() {
    let json_path = rustdoc_json::Builder::default()
        .toolchain("nightly-2022-12-18")
        .manifest_path("crates/belly_core/Cargo.toml")
        .build()
        .unwrap();

    let f = File::open(&json_path)
        .unwrap_or_else(|_| panic!("Could not open {}", json_path.to_str().unwrap()));
    let rdr = BufReader::new(f);
    let crt: Crate = from_reader(rdr).unwrap_or_else(|e| panic!("Can't parse json: {e:?}"));
    let types = fetch_parsers(&crt);
    let mut type_names: Vec<_> = types.keys().collect();
    type_names.sort();
    let mut props = fetch_properties(&crt);
    props.extend(fetch_compound_properties(&crt));
    let mut categories = HashMap::new();
    for prop in props {
        categories
            .entry(prop.category().unwrap_or("Custom".into()))
            .or_insert_with(|| vec![])
            .push(prop)
    }
    categories
        .values_mut()
        .for_each(|v| v.sort_by_key(|i| i.name.clone()));
    let mut category_keys: Vec<_> = categories.keys().cloned().collect();
    category_keys.sort_by(|a, b| match (a.as_str(), b.as_str()) {
        ("General", _) => std::cmp::Ordering::Less,
        (_, "General") => std::cmp::Ordering::Greater,
        ("Custom", _) => std::cmp::Ordering::Greater,
        (_, "Custom") => std::cmp::Ordering::Less,
        (a, b) => a.cmp(b),
    });
    println!("<!-- THIS DOC IS GENERATED FROM RUST DOCSTRINGS -->");
    println!("<!-- DO NOT EDIT IT BY HAND!!! -->");
    println!("# Reference");
    println!("| property | type | default |");
    println!("|----------|------|---------|");
    for category in category_keys.iter() {
        for prop in categories.get(category).unwrap().iter() {
            let prop_name = &prop.name;
            let prop_type = prop.prop_type.gen_linked_text().replace("|", "&#124;");
            let prop_def = prop.default.clone().unwrap_or("-".into());
            println!("|[`{prop_name}`](#property-{prop_name})|{prop_type}|`{prop_def}`|");
        }
    }
    println!("# Types");
    println!("");
    for type_name in type_names {
        let doc = types.get(type_name).unwrap();
        println!(r#"## <a name="{type_name}"></a>`{type_name}`"#);
        println!("");
        println!("{doc}");
    }
    println!("# Properties");
    for category in category_keys.iter() {
        println!("## {category}");
        for prop in categories.get(category).unwrap().iter() {
            println!(r#"### <a name="property-{0}"></a>`{0}`"#, prop.name);
            println!("type: {}", prop.prop_type.gen_linked_text());
            println!("");
            if let Some(default) = &prop.default {
                println!("default: `{default}`");
                println!("");
            }
            println!("{}", prop.docs.0)
        }
    }
}

fn fetch_parsers(crt: &Crate) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let parser_trait_id = crt
        .paths
        .iter()
        .filter(|(_, p)| p.kind == ItemKind::Trait)
        .map(|(i, p)| (i, p.path.join("::")))
        .filter(|(_, p)| p.as_str() == "belly_core::ess::property::PropertyParser")
        .map(|(i, _)| i)
        .next()
        .expect("No PropertyParser trait found");

    let parser_trait = crt
        .index
        .get(parser_trait_id)
        .expect("PropertyParser trait not declared in current crate");

    let ItemEnum::Trait(parser_trait) = &parser_trait.inner else {
        panic!("Property is not a trait")
    };

    for id in parser_trait.implementations.iter() {
        let Some(item) = crt.index.get(id) else {
            eprintln!("Looks like implementation defined in outer crate");
            continue;
        };
        let ItemEnum::Impl(imp) = &item.inner else {
            continue;
        };
        let Type::ResolvedPath(path) = &imp.for_ else {
            eprintln!(
                "Don't know how to handle PropertyParser implementation {:?}",
                imp.for_
            );
            continue;
        };
        let Some(impl_item) = crt.index.get(&path.id) else {
            eprintln!("Looks like {} implemented in outer crate", path.name);
            continue;
        };
        let Some(docs) = &impl_item.docs else {
            continue;
        };
        let Some(prop_type) = docattr("property-type", docs) else {
            continue;
        };
        if !prop_type.starts_with("$") {
            continue;
        }
        result.insert(prop_type.into(), docs.into());
    }
    result
}

fn fetch_widgets(crt: &Crate) -> Vec<Widget> {
    let widget_trait_id = crt
        .paths
        .iter()
        .filter(|(_, p)| p.kind == ItemKind::Trait)
        .map(|(i, p)| (i, p.path.join("::")))
        .filter(|(_, p)| p.as_str() == "belly_core::eml::build::Widget")
        .map(|(i, _)| i)
        .next()
        .expect("No Widget trait found");
    let mut impls = vec![];
    for item in crt.index.values() {
        if let ItemEnum::Impl(imp) = &item.inner {
            if let Some(path) = &imp.trait_ {
                if &path.id == widget_trait_id {
                    if let Type::ResolvedPath(impl_path) = &imp.for_ {
                        if let Some(widget_impl) = crt.index.get(&impl_path.id) {
                            impls.push(widget_impl);
                        } else {
                            eprintln!("Invalid crate for Widget implementation");
                        }
                    }
                }
            }
        }
    }

    let mut result = vec![];
    for item in impls {
        let mut all_extends = fetch_widget_extends(crt, item);
        let mut extends = None;
        while let Some(extends_item) = all_extends.pop() {
            let docs = Doc::new(extends_item.docs.as_ref().unwrap());
            let name = docs.attr("widget-name").unwrap().into();
            if let Some(module) = find_module(crt, extends_item) {
                extends = Some(Box::new(Widget {
                    docs,
                    name,
                    extends,
                    crt,
                    module,
                    links: extends_item.links.clone(),
                }))
            } else {
                break;
            }
        }
        let docs = if let Some(docs) = &item.docs {
            Doc::new(docs)
        } else {
            Doc::new("")
        };
        let Some(name) = docs.attr("widget-name") else {
            eprintln!("No @widget-name found for {}", item.name.as_ref().unwrap());
            continue;
        };
        if let Some(module) = find_module(crt, item) {
            result.push(Widget {
                name: name.into(),
                docs,
                extends,
                crt,
                module,
                links: item.links.clone(),
            })
        }
    }
    result
}

fn fetch_widget_extends<'a>(crt: &'a Crate, for_item: &Item) -> Vec<&'a Item> {
    let mut result = vec![];
    let widget_trait_id = crt
        .paths
        .iter()
        .filter(|(_, p)| p.kind == ItemKind::Trait)
        .map(|(i, p)| (i, p.path.join("::")))
        .filter(|(_, p)| p.as_str() == "belly_core::eml::build::Widget")
        .map(|(i, _)| i)
        .next()
        .expect("No Widget trait found");
    for item in crt.index.values() {
        if let ItemEnum::Impl(imp) = &item.inner {
            let Type::ResolvedPath(impl_for) = &imp.for_ else {
                continue;
            };
            if impl_for.id != for_item.id {
                continue;
            }
            if let Some(path) = &imp.trait_ {
                if &path.id == widget_trait_id {
                    let extends = imp
                        .items
                        .iter()
                        .filter_map(|i| crt.index.get(i))
                        .filter(|i| i.name == Some("Extends".into()))
                        .next()
                        .unwrap();
                    let ItemEnum::AssocType {
                        generics: _,
                        bounds: _,
                        default: Some(extends),
                    } = &extends.inner
                    else {
                        panic!("Expected assoc type")
                    };
                    let Type::ResolvedPath(extends) = extends else {
                        panic!("Expected ResolvedPath")
                    };
                    if let Some(extends) = crt.index.get(&extends.id) {
                        result.push(extends);
                        result.extend(fetch_widget_extends(crt, extends))
                    }
                }
            }
        }
    }
    result
}

fn find_module<'a>(crt: &'a Crate, for_item: &Item) -> Option<&'a Module> {
    for item in crt.index.values() {
        if let ItemEnum::Module(module) = &item.inner {
            if module
                .items
                .iter()
                .filter_map(|i| crt.index.get(i))
                .filter(|i| i.id == for_item.id)
                .next()
                .is_some()
            {
                return Some(module);
            }
        }
    }
    None
}

fn fetch_properties(crt: &Crate) -> Vec<Property> {
    let property_trait_id = crt
        .paths
        .iter()
        .filter(|(_, p)| p.kind == ItemKind::Trait)
        .map(|(i, p)| (i, p.path.join("::")))
        .filter(|(_, p)| p.as_str() == "belly_core::ess::property::Property")
        .map(|(i, _)| i)
        .next()
        .expect("No property trait found");
    let property_trait = crt
        .index
        .get(property_trait_id)
        .expect("Property trait not declared in current crate");
    let ItemEnum::Trait(property_trait) = &property_trait.inner else {
        panic!("Property is not a trait")
    };
    let mut result = vec![];
    for id in property_trait.implementations.iter() {
        let Some(item) = crt.index.get(id) else {
            eprintln!("Looks like implementation defined in outer crate");
            continue;
        };
        let ItemEnum::Impl(imp) = &item.inner else {
            continue;
        };
        let Type::ResolvedPath(path) = &imp.for_ else {
            eprintln!(
                "Don't know how to handle Property implementation {:?}",
                imp.for_
            );
            continue;
        };
        let Some(impl_item) = crt.index.get(&path.id) else {
            eprintln!("Looks like {} implemented in outer crate", path.name);
            continue;
        };
        let Some(parser_assoc) = imp
            .items
            .iter()
            .filter_map(|id| crt.index.get(id))
            .filter(|p| p.name.is_some())
            .filter(|p| p.name.as_ref().unwrap() == "Parser")
            .next()
        else {
            eprintln!("Can't find Parser associated type");
            continue;
        };
        let parser_path = if let ItemEnum::AssocType {
            generics: _,
            bounds: _,
            default,
        } = &parser_assoc.inner
        {
            let Some(default) = default else {
                eprintln!("Invalid Parser associated type");
                continue;
            };
            let Type::ResolvedPath(path) = default else {
                eprintln!("Invalid Parser associated type");
                continue;
            };
            path
        } else {
            eprintln!("Invalid Parser associated type");
            continue;
        };
        let Some(parser) = crt.index.get(&parser_path.id) else {
            eprint!("Parser implemented in outside crate");
            continue;
        };
        let prop_type = if let Some(docs) = &parser.docs {
            if let Some(prop_type) = docattr("property-type", docs) {
                PropertyType::Ref(prop_type.into())
            } else {
                PropertyType::Inline(docs.into())
            }
        } else {
            PropertyType::Unknown
        };

        let Some(docs) = &impl_item.docs else {
            eprintln!("Property without docs");
            continue;
        };
        let Some(prop_name) = docattr("property-name", docs) else {
            eprintln!("Missed @property-name doc attribute");
            continue;
        };
        let Some(prop_default) = docattr("property-default", docs) else {
            eprintln!("Missed @property-default doc attribute");
            continue;
        };

        result.push(Property {
            name: prop_name.into(),
            prop_type,
            default: Some(prop_default.into()),
            docs: Doc::new(docs),
        });
    }
    result
}

fn fetch_compound_properties(crt: &Crate) -> Vec<Property> {
    let property_trait_id = crt
        .paths
        .iter()
        .filter(|(_, p)| p.kind == ItemKind::Trait)
        .map(|(i, p)| (i, p.path.join("::")))
        .filter(|(_, p)| p.as_str() == "belly_core::ess::property::CompoundProperty")
        .map(|(i, _)| i)
        .next()
        .expect("No CompoundProperty trait found");
    let property_trait = crt
        .index
        .get(property_trait_id)
        .expect("CompoundProperty trait not declared in current crate");
    let ItemEnum::Trait(property_trait) = &property_trait.inner else {
        panic!("CompoundProperty is not a trait")
    };
    let mut result = vec![];
    for id in property_trait.implementations.iter() {
        let Some(item) = crt.index.get(id) else {
            eprintln!("Looks like CompoundProperty implementation defined in outer crate");
            continue;
        };
        let ItemEnum::Impl(imp) = &item.inner else {
            continue;
        };
        let Type::ResolvedPath(path) = &imp.for_ else {
            eprintln!(
                "Don't know how to handle CompoundProperty implementation {:?}",
                imp.for_
            );
            continue;
        };
        let Some(impl_item) = crt.index.get(&path.id) else {
            eprintln!(
                "Looks like CompoundProperty for {} implemented in outer crate",
                path.name
            );
            continue;
        };
        let docs = if let Some(docs) = &impl_item.docs {
            Doc::new(docs)
        } else {
            Doc::new("")
        };
        let prop_type = if let Some(pt) = docs.attr("property-type") {
            PropertyType::Inline(pt.into())
        } else {
            eprintln!(
                "Can't fetch @property-type from {} docs",
                impl_item.name.as_ref().unwrap()
            );
            continue;
        };
        let Some(prop_name) = docs.attr("property-name") else {
            eprintln!(
                "Can't fetch @property-name from {} docs",
                impl_item.name.as_ref().unwrap()
            );
            continue;
        };
        result.push(Property {
            name: prop_name.into(),
            prop_type,
            default: None,
            docs,
        })
    }
    result
}

struct Widget<'a> {
    crt: &'a Crate,
    links: HashMap<String, Id>,
    name: String,
    #[allow(dead_code)]
    module: &'a Module,
    extends: Option<Box<Widget<'a>>>,
    docs: Doc,
}

impl<'a> Widget<'a> {
    pub fn docs_body(&self) -> String {
        let body = if let Some(body) = self.docs.block("widget-body") {
            body.trim()
        } else {
            ""
        };
        body.into()
    }

    pub fn docs_params(&self) -> String {
        let mut result = "".to_string();
        let mut widget = Some(self);
        while let Some(w) = widget {
            if let Some(params) = w.docs.block("widget-params") {
                let mut params = params.trim().to_string();
                for (link, id) in w.links.iter() {
                    if let Some(item) = self.crt.index.get(id) {
                        let docs = Doc::new(item.docs.as_ref().unwrap_or(&format!("")));
                        let item_name = link.trim_matches('`');
                        let inline = format!("<!-- @inline {item_name} -->");
                        params = params.replace(
                            &inline,
                            docs.0.as_str().trim().replace("\n", "\n  ").as_str(),
                        );
                    }
                    // TODO: replace with valid link to crate
                    let type_link = format!("[{link}]");
                    params = params.replace(&type_link, link);
                }
                if !params.is_empty() {
                    if w.name != self.name {
                        result += format!("\nfrom `<{}>`\n", w.name).as_str();
                    }
                    result += params.as_str();
                }
                widget = w.extends.as_ref().map(|b| b.as_ref())
            }
        }
        result.trim().into()
    }
}

enum PropertyType {
    Unknown,
    Inline(String),
    Ref(String),
}

impl PropertyType {
    pub fn gen_linked_text(&self) -> String {
        let source = match self {
            Self::Unknown => return "unknown".into(),
            Self::Inline(i) => i,
            Self::Ref(r) => r,
        };
        source
            .split("|")
            .map(|word| {
                if word.starts_with("$") {
                    "[`".to_string() + word + "`](#" + word + ")"
                } else {
                    "`".to_string() + word + "`"
                }
            })
            .collect::<Vec<_>>()
            .join("**|**")
    }
}

impl std::fmt::Display for PropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Ref(r) => write!(f, "{r}"),
            Self::Inline(i) => write!(f, "{i}"),
        }
    }
}

struct Doc(String);

impl Doc {
    fn new<T: AsRef<str>>(value: T) -> Self {
        Doc(value.as_ref().to_string()).alter()
    }
    fn attr(&self, name: &str) -> Option<&str> {
        docattr(name, self.0.as_str())
    }
    fn block(&self, name: &str) -> Option<&str> {
        docblock(name, self.0.as_str())
    }
    fn alter(&self) -> Doc {
        Doc(docalter(self.0.as_str()))
    }
    #[allow(dead_code)]
    fn block_replace<F: Fn(&str) -> String>(&self, name: &str, repl: F) -> String {
        docblock_replace(name, self.0.as_str(), repl)
    }
}

pub struct Property {
    name: String,
    prop_type: PropertyType,
    // CompoundProperty does not have default value
    default: Option<String>,
    docs: Doc,
}

impl Property {
    fn category(&self) -> Option<String> {
        self.docs.attr("property-category").map(|v| v.to_string())
    }
}

fn docattr<'a>(name: &str, mut docstring: &'a str) -> Option<&'a str> {
    while let Some(idx) = docstring.find("<!--") {
        docstring = &docstring[idx + 4..];
        docstring = docstring.trim_start();
        let Some(item) = docstring.strip_prefix("@") else {
            continue;
        };
        let Some(item) = item.strip_prefix(name) else {
            continue;
        };
        let Some(item) = item.trim_start().strip_prefix("=") else {
            continue;
        };
        let Some(idx) = item.find("-->") else {
            continue;
        };
        return Some(&item[0..idx].trim());
    }
    None
}

fn docalter<'a>(mut docstring: &'a str) -> String {
    let mut result = String::new();
    while let Some(idx) = docstring.find(format!("<!-- @alter").as_str()) {
        result = result + &docstring[..idx];
        docstring = &docstring[idx..];
        docstring = docstring.strip_prefix("<!-- @alter").unwrap();
        let next = docstring.find("-->").unwrap();
        result = result + &docstring[..next];
        docstring = &docstring[next..];
        docstring = docstring.strip_prefix("-->").unwrap();
    }
    if result.is_empty() {
        docstring.to_string()
    } else {
        result
    }
}

fn docblock<'a>(name: &str, mut docstring: &'a str) -> Option<&'a str> {
    while let Some(idx) = docstring.find(format!("<!-- @{name}-begin").as_str()) {
        docstring = &docstring[idx..];
        let start = docstring.find("-->").unwrap() + 3;
        let end = docstring
            .find(format!("<!-- @{name}-end").as_str())
            .unwrap();
        let found: &str = &docstring[start..end];
        return Some(found);
    }
    None
}

fn docblock_replace<'a, F: Fn(&'a str) -> String>(
    name: &str,
    docstring: &'a str,
    replace: F,
) -> String {
    while let Some(idx) = docstring.find(format!("<!-- @{name}-begin").as_str()) {
        let docmatch = &docstring[idx..];
        let start = docmatch.find("-->").unwrap() + 3;
        let end = docmatch.find(format!("<!-- @{name}-end").as_str()).unwrap();
        let found: &str = &docmatch[start..end];
        let docmatch = &docmatch[end..];
        let last = docmatch.find("-->").unwrap() + 3;

        return docstring[..idx].to_string() + replace(found).as_str() + &docmatch[last..];
    }
    docstring.to_string()
}
