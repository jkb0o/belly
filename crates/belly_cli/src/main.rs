use std::{collections::HashMap, fs::File, io::BufReader};

use rustdoc_json;
use rustdoc_types::{Crate, ItemEnum, ItemKind, Type};
use serde_json::from_reader;

fn main() {
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
        let ItemEnum::Impl(imp) = &item.inner else { continue };
        let Type::ResolvedPath(path) = &imp.for_ else {
            eprintln!("Don't know how to handle PropertyParser implementation {:?}", imp.for_);
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
    // println!("Found trait: {property_trait:?}");
    let ItemEnum::Trait(property_trait) = &property_trait.inner else {
        panic!("Property is not a trait")
    };
    let mut result = vec![];
    for id in property_trait.implementations.iter() {
        let Some(item) = crt.index.get(id) else {
            eprintln!("Looks like implementation defined in outer crate");
            continue;
        };
        let ItemEnum::Impl(imp) = &item.inner else { continue };
        let Type::ResolvedPath(path) = &imp.for_ else {
            eprintln!("Don't know how to handle Property implementation {:?}", imp.for_);
            continue;
        };
        let Some(impl_item) = crt.index.get(&path.id) else {
            eprintln!("Looks like {} implemented in outer crate", path.name);
            continue;
        };
        // println!("{}", path.name);
        let Some(parser_assoc) = imp.items
            .iter()
            .filter_map(|id| crt.index.get(id))
            .filter(|p| p.name.is_some())
            .filter(|p| p.name.as_ref().unwrap() == "Parser")
            .next() else {
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
            continue
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
    // println!("Found trait: {property_trait:?}");
    let ItemEnum::Trait(property_trait) = &property_trait.inner else {
        panic!("CompoundProperty is not a trait")
    };
    let mut result = vec![];
    for id in property_trait.implementations.iter() {
        let Some(item) = crt.index.get(id) else {
            eprintln!("Looks like CompoundProperty implementation defined in outer crate");
            continue;
        };
        let ItemEnum::Impl(imp) = &item.inner else { continue };
        let Type::ResolvedPath(path) = &imp.for_ else {
            eprintln!("Don't know how to handle CompoundProperty implementation {:?}", imp.for_);
            continue;
        };
        let Some(impl_item) = crt.index.get(&path.id) else {
            eprintln!("Looks like CompoundProperty for {} implemented in outer crate", path.name);
            continue;
        };
        let docs = if let Some(docs) = &impl_item.docs {
            Doc(docs.into())
        } else {
            Doc("".into())
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
            eprintln!("Can't fetch @property-name from {} docs", impl_item.name.as_ref().unwrap());
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
        Doc(value.as_ref().to_string())
    }
    fn attr(&self, name: &str) -> Option<&str> {
        docattr(name, self.0.as_str())
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
