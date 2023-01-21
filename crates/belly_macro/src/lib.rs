use proc_macro2::{Span, TokenStream};
use quote::*;
extern crate proc_macro;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Error, Expr, ExprPath};
use syn_rsx::{parse, Node, NodeAttribute, NodeElement};
use toml;

fn core_path() -> TokenStream {
    let default_path = quote! { ::belly_core };
    let Some(manifest_path) = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|mut path| { path.push("Cargo.toml"); path })
        else { return default_path };
    let Ok(manifest) = std::fs::read_to_string(&manifest_path) else {
        return default_path
    };
    let Ok(manifest) = toml::from_str::<toml::map::Map<String, toml::Value>>(&manifest) else {
        return default_path
    };

    let Some(pkg) = manifest.get("package") else { return default_path };
    let Some(pkg) = pkg.as_table() else { return default_path };
    let Some(pkg) = pkg.get("name") else { return default_path };
    let Some(pkg) = pkg.as_str() else { return default_path };
    let path = if pkg.trim() == "belly_widgets" {
        quote! { ::belly_core }
    } else {
        quote! { ::belly::core }
    };
    path
}

enum Param {
    Direct(syn::Ident),
    Proxy(syn::Ident, syn::Type, syn::Ident),
}

impl syn::parse::Parse for Param {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![:]>()?;
        let prop_type = input.parse::<syn::Type>()?;
        let target = if input.is_empty() {
            name.clone()
        } else {
            input.parse::<syn::Token![=>]>()?;
            input.parse::<syn::Ident>()?
        };
        Ok(Param::Proxy(name, prop_type, target))
    }
}

impl Param {
    fn from_field(field: &syn::Field) -> syn::Result<Vec<Param>> {
        let span = field.span();
        let Some(field_ident) = field.ident.as_ref() else {
            return  Err(syn::Error::new(span, "Tuple Structs not yet supported by Widget derive."));
        };
        let mut result = vec![];
        for attr in field.attrs.iter().filter(|a| a.path.is_ident("param")) {
            if attr.tokens.is_empty() {
                result.push(Param::Direct(field_ident.clone()));
            } else {
                result.push(attr.parse_args::<Param>()?);
            }
        }
        Ok(result)
    }
}

enum Extend {
    Styles(syn::Ident),
    Descriptor(syn::Ident),
    StylesAndDescriptor(syn::Ident, syn::Ident),
}

impl Extend {
    fn span(&self) -> Span {
        match self {
            Extend::Styles(ident) => ident.span(),
            Extend::Descriptor(ident) => ident.span(),
            Extend::StylesAndDescriptor(ident, _) => ident.span(),
        }
    }
    fn descriptor(&self) -> Option<syn::Ident> {
        match self {
            Extend::Descriptor(descriptor) | Extend::StylesAndDescriptor(_, descriptor) => {
                Some(descriptor.clone())
            }
            _ => None,
        }
    }
    fn styles(&self) -> Option<syn::Ident> {
        match self {
            Extend::Styles(styles) | Extend::StylesAndDescriptor(styles, _) => Some(styles.clone()),
            _ => None,
        }
    }

    fn from_attributes(attrs: &Vec<syn::Attribute>) -> syn::Result<Option<Extend>> {
        let extends: Vec<_> = attrs
            .iter()
            .filter(|a| a.path.is_ident("extends"))
            .map(|a| a.parse_args::<Extend>())
            .collect();
        let mut found = None;
        for extend in extends {
            let extend = extend?;
            found = match (found, extend) {
                (None, extend) => Some(extend),
                (Some(Extend::Styles(styles)), Extend::Descriptor(descriptor))
                | (Some(Extend::Descriptor(descriptor)), Extend::Styles(styles)) => {
                    Some(Extend::StylesAndDescriptor(styles, descriptor))
                }
                (Some(found), _) => {
                    return Err(syn::Error::new(
                        found.span(),
                        format!("Invalid #[extends] sequence."),
                    ))
                }
            }
        }
        Ok(found)
    }
}

impl syn::parse::Parse for Extend {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let mut styles: Option<syn::Ident> = None;
        let mut descriptor: Option<syn::Ident> = None;
        while !input.is_empty() {
            let kind = input.parse::<syn::Ident>()?;
            let target = if &kind.to_string() == "descriptor" {
                &mut descriptor
            } else if &kind.to_string() == "styles" {
                &mut styles
            } else {
                return Err(syn::Error::new(
                    span,
                    "#[extend] support only `styles` and `descriptor` args",
                ));
            };
            input.parse::<syn::Token![=]>()?;
            let value = input.parse::<syn::Ident>()?;
            *target = Some(value);
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }
        match (styles, descriptor) {
            (Some(styles), None) => Ok(Extend::Styles(styles)),
            (None, Some(descriptor)) => Ok(Extend::Descriptor(descriptor)),
            (Some(styles), Some(descriptor)) => Ok(Extend::StylesAndDescriptor(styles, descriptor)),
            _ => Err(syn::Error::new(
                span,
                "#[extend] should provide at least one argument: `styles` and/or `descriptor`",
            )),
        }
    }
}

trait FieldExt {
    fn is_entity(&self) -> bool;
}

impl FieldExt for syn::Field {
    fn is_entity(&self) -> bool {
        let syn::Type::Path(path) = &self.ty else {
            return false;
        };
        let Some(last) = path.path.segments.iter().last() else {
            return false;
        };
        &last.ident.to_string() == "Entity"
    }
}

fn create_single_command_stmt(expr: &ExprPath) -> TokenStream {
    let component_span = expr.span();
    if let Some(component) = expr.path.get_ident() {
        if component.to_string().chars().next().unwrap().is_uppercase() {
            quote_spanned! {component_span=>
                c.insert(#component::default());
            }
        } else {
            quote_spanned! {component_span=>
                c.insert(#component);
            }
        }
    } else {
        Error::new(component_span, "Invalid components declaration").into_compile_error()
    }
}

fn create_command_stmts(expr: &Expr) -> TokenStream {
    let core = core_path();
    let with_body = match expr {
        Expr::Path(path) => create_single_command_stmt(path),
        Expr::Tuple(components) => {
            let mut components_expr = quote! {};
            for component_expr in components.elems.iter() {
                let component_span = component_expr.span();
                if let Expr::Path(component) = component_expr {
                    let component_expr = create_single_command_stmt(component);
                    components_expr = quote_spanned! {component_span=>
                        #components_expr
                        #component_expr
                    };
                } else {
                    return Error::new(component_span, "Invalid component name")
                        .into_compile_error();
                }
            }
            components_expr
        }
        _ => {
            return Error::new(expr.span(), "Invalid components declaration").into_compile_error();
        }
    };
    let expr_span = expr.span();
    quote_spanned! {expr_span=>
        __ctx.params.add(#core::eml::Param::from_commands("with", ::std::boxed::Box::new(move |c| {
            #with_body
        })));
    }
}

fn create_attr_stmt(attr: &NodeAttribute) -> TokenStream {
    let core = core_path();
    let attr_name = attr.key.to_string();
    match &attr.value {
        None => {
            return quote! {
                __ctx.params.add(#core::eml::Param::new(
                    #attr_name.into(),
                    #core::build::Variant::Bool(true)
                ));
            };
        }
        Some(attr_value) => {
            let attr_value = attr_value.as_ref();
            let attr_span = attr_value.span();
            if attr_name == "with" {
                create_command_stmts(attr_value)
            } else if attr_name == "params" {
                quote_spanned! {attr_span=>
                    __ctx.params.merge(#attr_value);
                }
            } else {
                quote_spanned! {attr_span=>
                    __ctx.params.add(#core::eml::Param::new(
                        #attr_name.into(),
                        (#attr_value).into()
                    ));
                }
            }
        }
    }
}

fn process_for_loop(node: &NodeElement) -> TokenStream {
    let span = node.span();
    if node.attributes.len() != 2 {
        return err2(
            span,
            "<for> tag should have exactly 2 attributes: <for item in=iter>",
        );
    }
    let Node::Attribute(item_attr) = &node.attributes[0] else {
        return err2(span, "Can't threat node ast Node::Attribute")
    };
    if item_attr.value.is_some() {
        return err2(
            span,
            "The first attribute of <for> tag shouldn't has any value: <for item in=iter>",
        );
    }
    let item_ident = syn::Ident::new(&item_attr.key.to_string(), item_attr.span());
    let Node::Attribute(iter_attr) = &node.attributes[1] else {
        return err2(span, "Can't threat node as Node::Attribute")
    };
    if iter_attr.value.is_none() {
        return err2(
            span,
            "The second attribute of <for> tag shold has some value: <for item in=iter>",
        );
    }
    let iter_value = iter_attr.value.as_ref().unwrap().as_ref();

    let mut loop_content = quote! {};
    for ch in node.children.iter() {
        if let Node::Element(elem) = ch {
            if &elem.name.to_string() == "for" {
                let expr = process_for_loop(elem);
                loop_content = quote! {
                    #loop_content
                    #expr
                };
                continue;
            }
        }

        let expr = walk_nodes(ch, true);
        loop_content = quote! {
            #loop_content
            __ctx.children.push( #expr );
        }
    }
    quote! {
        for #item_ident in #iter_value {
            #loop_content
        }
    }
}

fn process_slots(node: &NodeElement) -> TokenStream {
    let core = core_path();
    let span = node.span();
    if node.attributes.len() != 1 {
        return err2(
            span,
            "<slot> tag should have exactly 1 attribute: <slot grabber> or <slot name=\"grabber\">",
        );
    }
    let Node::Attribute(attr) = &node.attributes[0] else {
        return err2(span, "Can't threat node ast Node::Attribute")
    };
    let mut slot_content = quote! {};
    for ch in node.children.iter() {
        let expr = walk_nodes(ch, true);
        slot_content = quote! {
            #slot_content
            __slot_value.push( #expr );
        }
    }
    if attr.value.is_none() {
        let slot_name = attr.key.to_string();
        quote! {
            let mut __slot_value: Vec<Entity> = vec![];
            #slot_content
            __world.resource::<#core::eml::build::Slots>()
                .clone()
                .insert(#core::tagstr::Tag::new(#slot_name), __slot_value);
        }
    } else {
        if &attr.key.to_string() != "define" {
            return err2(
                span,
                "<slot> definition should have define attribute: <slot define=\"grabber\">",
            );
        }
        let slot_name = attr.value.as_ref().unwrap().as_ref();
        quote! {
            let __slot_value = __world.resource::<#core::eml::build::Slots>()
                .clone()
                .remove(#core::tagstr::Tag::new(#slot_name));
            if let Some(__slot_value) = __slot_value {
                __ctx.children.extend(__slot_value);
            } else {
                let mut __slot_value: Vec<Entity> = vec![];
                #slot_content
                __ctx.children.extend(__slot_value);
            }
        }
    }
}

fn walk_nodes<'a>(element: &'a Node, create_entity: bool) -> TokenStream {
    let core = core_path();
    let mut children = quote! {};
    let mut connections = quote! {};
    let mut parent = if create_entity {
        quote! { let __parent = __world.spawn_empty().id(); }
    } else {
        quote! {}
    };
    if let Node::Element(element) = element {
        let mut parent_defined = false;
        for attr in element.attributes.iter() {
            if let Node::Block(entity) = attr {
                let entity_span = entity.value.span();
                let entity = entity.value.as_ref();
                if parent_defined {
                    return Error::new(entity_span, "Entity already provided by entity attribute")
                        .into_compile_error();
                }
                parent_defined = true;
                parent = quote! {
                    let __parent = #entity;
                };
            } else if let Node::Attribute(attr) = attr {
                let attr_name = attr.key.to_string();
                let attr_span = attr.span();
                if let Some(signal) = attr_name.strip_prefix("on:") {
                    let Some(connection) = attr.value.as_ref() else {
                        return Error::new(attr_span, format!("on:{signal} param should provide connection"))
                            .into_compile_error();
                    };
                    let connection = connection.as_ref();
                    let signal_ident = syn::Ident::new(signal, connection.span());
                    connections = quote_spanned! {attr_span=>
                        #connections
                        __builder.#signal_ident(__world, __parent, #connection);
                    }
                } else if let Some(prop) = attr_name.strip_prefix("bind:") {
                    let Some(bind) = attr.value.as_ref() else {
                        return Error::new(attr_span, format!("bind:{prop} param should provide connection"))
                            .into_compile_error();
                    };
                    let bind = bind.as_ref();
                    let stream = bind.to_token_stream().to_string();
                    if stream.trim().starts_with("to!") {
                        let bind_from = format!("bind_from_{prop}");
                        let bind_from = syn::Ident::new(&bind_from, bind.span());
                        connections = quote_spanned! {attr_span=>
                            #connections
                            (__builder.#bind_from(__parent) >> #bind).write(__world);
                        };
                    } else if stream.trim().starts_with("from!") {
                        let bind_to = format!("bind_to_{prop}");
                        let bind_to = syn::Ident::new(&bind_to, bind.span());
                        connections = quote_spanned! {attr_span=>
                            #connections
                            (__builder.#bind_to(__parent) << #bind).write(__world);
                        };
                    }
                    // panic!("bind def: {}", bind_def.to_token_stream());
                    // let signal_ident = syn::Ident::new(signal, connection.span());
                    // connections = quote_spanned! {attr_span=>
                    //     #connections
                    //     __builder.#signal_ident(__world, __parent, #connection);
                    // }
                } else if &attr_name == "entity" {
                    if parent_defined {
                        return Error::new(attr_span, "Entity already provided by braced block")
                            .into_compile_error();
                    }
                    parent_defined = true;
                    let attr_value = attr.value.as_ref();
                    if attr_value.is_none() {
                        return Error::new(attr_span, "Attriute entity should has a value")
                            .into_compile_error();
                    }
                    let entity = attr_value.unwrap().as_ref();
                    parent = quote_spanned! { attr_span=>
                        let __parent = #entity;
                    };
                } else {
                    let attr_stmt = create_attr_stmt(attr);
                    children = quote! {
                        #children
                        #attr_stmt
                    };
                }
            }
        }
        for child in element.children.iter() {
            match child {
                Node::Element(element) => {
                    let element_name = element.name.to_string();
                    let expr = match element_name.as_str() {
                        "for" => process_for_loop(element),
                        "slot" => process_slots(element),
                        _ => {
                            let expr = walk_nodes(child, true);
                            quote! {
                                __ctx.children.push( #expr );
                            }
                        }
                    };
                    children = quote! {
                        #children
                        #expr
                    }
                }
                Node::Text(text) => {
                    let text = text.value.as_ref();
                    children = quote! {
                        #children
                        __ctx.children.push(
                            __world.spawn(::bevy::prelude::TextBundle {
                                text: ::bevy::prelude::Text::from_section(
                                    #text,
                                    ::std::default::Default::default()
                                ),
                                ..default()
                            })
                            .insert(#core::build::Element::inline())
                            .id()
                        );
                    };
                }
                Node::Block(block) => {
                    let block = block.value.as_ref();
                    let block_span = block.span();
                    children = quote_spanned! { block_span=>
                        #children
                        let __node = __world.spawn_empty().id();
                        for __child in #block.into_content(__node, __world).iter() {
                            __ctx.children.push( __child.clone() );
                        }
                    }
                }
                _ => (),
            };
        }

        let tag = syn::Ident::new(&element.name.to_string(), element.span());
        quote! {
            {
                #parent
                let mut __ctx = #core::build::ElementContextData::new(__parent);

                #children
                let __builder = #core::Widgets::#tag();
                __builder.get_builder().build(__world, __ctx);
                #connections
                __parent
            }
        }
    } else {
        quote! {}
    }
}

#[proc_macro]
pub fn eml(tree: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let core = core_path();
    match parse(tree.into()) {
        Err(err) => err.to_compile_error().into(),
        Ok(nodes) => {
            let body = walk_nodes(&nodes[0], false);
            // nodes[0]
            let wraped = quote! {
                #core::build::ElementsBuilder::new(
                    move |
                        __world: &mut ::bevy::prelude::World,
                        __parent: ::bevy::prelude::Entity
                    | {
                        let mut __slots_resource = __world.resource::<#core::eml::build::Slots>().clone();
                        let __defined_slots = __slots_resource.keys();
                        #body;
                        for __slot in __slots_resource.keys() {
                            if !__defined_slots.contains(&__slot) {
                                warn!("Detected unused slot '{}', despawning it contnent.", __slot);
                                use ::bevy::ecs::system::Command;
                                for __entity in __slots_resource.remove(__slot).unwrap() {
                                    let __despawn =  ::bevy::prelude::DespawnRecursive {
                                        entity: __entity
                                    };
                                    __despawn.write(__world);
                                }
                            }
                        }
                    }
            )};
            proc_macro::TokenStream::from(wraped)
        }
    }
}

fn err2(span: Span, message: &str) -> TokenStream {
    return syn::Error::new(span, message).into_compile_error();
}
fn err(span: Span, message: &str) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(syn::Error::new(span, message).to_compile_error())
}

#[proc_macro_derive(Widget, attributes(alias, param, signal, bindto, bindfrom, extends))]
pub fn widget_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let core = core_path();
    let ast = parse_macro_input!(input as DeriveInput);
    let span = ast.span();

    let component = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let component_str = format!("{component}");
    let mut names_expr = quote! { #component_str };
    let mod_descriptor = format_ident!("{}_widget_descriptor", component_str.to_lowercase());
    let extension_ident = format_ident!("{component}WidgetExtension");

    // TODO: use `doclines` to generate XSD docs like extension docs
    let (_doclines, docs) = parse_docs(&ast.attrs);
    let mut bind_body = quote! {
        let this = ctx.entity();
    };
    let mut extension_body = quote! {
        #[allow(non_snake_case)]
        fn #component() -> #mod_descriptor::Descriptor {
            #mod_descriptor::Descriptor
        }
    };

    let syn::Data::Struct(data) = &ast.data else {
        return err(span, "Widget could be derived only for structs");
    };

    for field in data.fields.iter() {
        let Some(field_ident) = field.ident.as_ref() else {
            return err(span, "Tuple Structs not yet supported by Widget derive.")
        };
        if field.is_entity() {
            bind_body = quote! {
                #bind_body
                let #field_ident = self.#field_ident;
            };
        }
    }

    for field in data.fields.iter() {
        let Some(field_ident) = field.ident.as_ref() else {
            return  err(span, "Tuple Structs not yet supported by Widget derive.");
        };

        let field_name = format!("{field_ident}");
        for attr in field.attrs.iter() {
            let attr_name = attr.path.get_ident().unwrap().to_string();
            if attr.path.is_ident("bindto") || attr.path.is_ident("bindfrom") {
                let bind_type = attr_name.strip_prefix("bind").unwrap();
                let Ok(bind) = attr.parse_args::<TokenStream>() else {
                    return err(attr.span(), "#[bind{bind_type}] attribute should be defined with to! or from! macro: #[bind{bind_type}(entity, Component:field)]")
                };
                // panic!("{bind}")
                bind_body = if bind_type == "to" {
                    let bind_from = format_ident!("bind_from_{field_name}");
                    let bind_from = quote! {
                        #mod_descriptor::Descriptor::get_instance().#bind_from(this)
                    };
                    quote! {
                        #bind_body
                        ctx.commands().add(move |world: &mut ::bevy::prelude::World| {
                            (#core::to!(#bind) << #bind_from).write(world);
                        });
                    }
                } else {
                    let bind_to = format_ident!("bind_to_{field_name}");
                    let bind_to = quote! {
                        #mod_descriptor::Descriptor::get_instance().#bind_to(this)
                    };
                    quote! {
                        #bind_body
                        ctx.commands().add(move |world: &mut ::bevy::prelude::World| {
                            (#core::from!(#bind) >> #bind_to).wrote(world);
                        });
                    }
                }
            }
        }
    }

    for attr in ast.attrs.iter() {
        if attr.path.is_ident("alias") {
            let Ok(alias) = attr.parse_args::<syn::Ident>() else {
                return err(attr.span(), "Alias should be defined using tokens: `#[alias(alias_name)]");
            };
            let alias_str = format!("{alias}");
            names_expr = quote! { #names_expr, #alias_str };
            extension_body = quote! {
                #extension_body
                #docs
                fn #alias() -> #mod_descriptor::Descriptor {
                    #mod_descriptor::Descriptor
                }
            }
        }
    }

    let connect_signals = match parse_signals(&ast.attrs) {
        Ok(tokens) => tokens,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    let bind_descriptors = match parse_binds(&ast) {
        Ok(tokens) => tokens,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    let construct_body = match prepare_construct_instance(&ast) {
        Ok(tokens) => tokens,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };
    let aliases_decl = match prepare_extends_aliases(&ast.attrs) {
        Ok(tokens) => tokens,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };
    let extends_decl = match prepare_extends_descriptor(&component, &ast.attrs) {
        Ok(tokens) => tokens,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };
    // panic!("binds: {}", bind_descriptors.to_string());

    proc_macro::TokenStream::from(quote! {
        mod #mod_descriptor {
            use super::*;
            pub struct Descriptor;
            impl Descriptor {
                pub fn get_instance() -> &'static Descriptor {
                    &&Descriptor
                }

                pub fn get_builder(&self) -> #core::build::ElementBuilder {
                    #component::as_builder()
                }
                #connect_signals

                #bind_descriptors

            }

            #extends_decl
        }
        impl #impl_generics #core::build::Widget for #component #ty_generics #where_clause {
            fn names() -> &'static [&'static str] {
                &[#names_expr]
            }

            #aliases_decl

            fn construct_component(world: &mut ::bevy::prelude::World, params: &mut #core::eml::Params) -> ::std::option::Option<Self> {
                ::std::option::Option::Some(#component {
                    #construct_body
                })
            }
            #[allow(unused_variables)]
            fn bind_component(&mut self, ctx: &mut #core::build::ElementContext) {
                #bind_body
            }
        }

        pub trait #extension_ident {
            type Descriptor;
            fn descriptor() -> Self::Descriptor;
            #extension_body
        }

        impl #extension_ident for #core::Widgets {
            type Descriptor = #mod_descriptor::Descriptor;
            fn descriptor() -> Self::Descriptor {
                #mod_descriptor::Descriptor
            }
        }
    })
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn prepare_construct_instance(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let core = core_path();
    let mut construct_body = quote! {};
    let span = ast.span();
    let syn::Data::Struct(data) = &ast.data else {
        return Err(syn::Error::new(span, "Widget could be derived only for structs"));
    };
    for field in data.fields.iter() {
        let span = field.span();
        let Some(field_ident) = field.ident.as_ref() else {
            return Err(syn::Error::new(span, "Tuple Structs not yet supported by Widget derive."))
        };
        let field_str = format!("{field_ident}");
        if &field_str == "ctx" || &field_str == "this" {
            return Err(syn::Error::new(
                span,
                format!(
                    "Using `{field_str}` as field name may lead to Widget's unexpected behaviour."
                ),
            ));
        }
        let params = Param::from_field(&field)?;
        if params.len() == 0 {
            construct_body = if field.is_entity() {
                quote! {
                    #construct_body
                    #field_ident: world.spawn_empty().id(),
                }
            } else {
                quote! {
                    #construct_body
                    #field_ident:  ::std::default::Default::default(),
                }
            };
            continue;
        }
        let mut proxy_body = quote! {};
        for param in params {
            match param {
                Param::Direct(ident) => {
                    let param_str = format!("{ident}");
                    construct_body = quote! {
                        #construct_body
                        #ident: #core::eml::build::FromWorldAndParam::from_world_and_param(
                            world, params.drop_variant(#core::Tag::new(#param_str)).unwrap_or(
                                #core::build::Variant::Undefined
                            )
                        ),
                    };
                }
                Param::Proxy(ident, _param_type, param_target) => {
                    let param_str = format!("{ident}");
                    let target_str = format! {"{param_target}"};
                    proxy_body = quote! {
                        #proxy_body
                        if let Some(param) = params.drop_variant(#core::Tag::new(#param_str)) {
                            proxy_params.insert(#target_str, param);
                        }
                    };
                }
            }
        }
        if !proxy_body.is_empty() {
            construct_body = quote! {
                #construct_body
                #field_ident: #core::eml::build::FromWorldAndParam::from_world_and_param(world, #core::build::Variant::Params({
                    let mut proxy_params = #core::eml::Params::default();
                    #proxy_body
                    proxy_params
                })),
            }
        }
    }
    Ok(quote! {
        #construct_body
    })
}

fn parse_binds(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let core = core_path();
    let component = &ast.ident;
    let span = ast.span();
    let mut binds = quote! {};
    let syn::Data::Struct(data) = &ast.data else {
        return Err(syn::Error::new(span, "Widget could be derived only for structs"));
    };
    for field in data.fields.iter() {
        let Some(target_field) = field.ident.as_ref() else {
            return  Err(syn::Error::new(span, "Tuple Structs not yet supported by Widget derive."));
        };
        // let params = match Param::from_field(field) {
        //     Ok(params) => params,
        //     Err(err) =>
        // }
        for param in Param::from_field(field)? {
            // let (param, setter, getter) = if let Ok(field) = attr.parse_args::<syn::Ident>() {
            //     (field.clone(), quote!(|#field), quote!(.#field()))
            // } else {
            //     (target_field.clone(), quote! {}, quote!{})
            // };
            let field_type = &field.ty;
            let target = match &param {
                Param::Direct(ident) => ident,
                Param::Proxy(ident, _, _) => ident,
            };
            let (bind_to_type, bind_to_type_params, bind_to_transformer) = match &param {
                Param::Direct(_ident) => (
                    quote! { ToComponentWithoutTransformer },
                    quote! { #field_type },
                    quote! {},
                ),
                Param::Proxy(_ident, param_type, param_target) => (
                    quote! { ToComponent },
                    quote! { #param_type, #field_type },
                    quote! { |#param_target },
                ),
            };
            let (from_type, from_getter) = match &param {
                Param::Direct(_) => (field_type, quote! {}),
                Param::Proxy(_ident, param_type, param_target) => {
                    (param_type, quote! { .#param_target() })
                }
            };
            let bind_to_ident = format_ident!("bind_to_{target}");
            let bind_from_ident = format_ident!("bind_from_{target}");
            binds = quote! {
                #binds

                pub fn #bind_to_ident(&self, target: ::bevy::prelude::Entity)
                -> #core::relations::bind::#bind_to_type<#component, #bind_to_type_params>
                {
                    #core::to!(target, #component:#target_field #bind_to_transformer)
                }

                pub fn #bind_from_ident(&self, source: ::bevy::prelude::Entity)
                -> #core::relations::bind::FromComponent<#component, #from_type>
                {
                    #core::from!(source, #component:#target_field #from_getter)
                }
            };
        }
    }
    Ok(binds)
}

fn parse_docs(attrs: &Vec<syn::Attribute>) -> (Vec<String>, TokenStream) {
    let mut docs = quote! {};
    let mut doclines = vec![];

    for attr in attrs.iter().filter(|a| a.path.is_ident("doc")) {
        let docline = attr.tokens.to_string();
        if docline.starts_with("= ") {
            doclines.push(docline[2..].to_string());
        }
        docs = quote! {
            #docs
            #attr
        }
    }
    (doclines, docs)
}

fn parse_signals(attrs: &Vec<syn::Attribute>) -> syn::Result<TokenStream> {
    let core = core_path();
    let mut connect_body = quote! {};
    for attr in attrs.iter() {
        if attr.path.is_ident("signal") {
            let span = attr.span();
            // let signal_decl = attr.tokens.clone();
            let Ok(meta) = attr.parse_meta() else {
                return Err(syn::Error::new(span,  "Invalid syntax fo #[signal(name, Event, filter)] attribute"));
            };
            let syn::Meta::List(signal_cfg) = meta else {
                return Err(syn::Error::new(span, "Invalid syntax fo #[signal(name, Event, filter)] attribute"));    
            };
            let signal_cfg: Vec<_> = signal_cfg.nested.iter().collect();
            if signal_cfg.len() != 3 {
                return Err(syn::Error::new(
                    span,
                    "Invalid syntax fo #[signal(name, Event, filter)] attribute.",
                ));
            }
            let syn::NestedMeta::Meta(name) = signal_cfg[0] else {
                let span = signal_cfg[0].span();
                return Err(syn::Error::new(span, "Expected ident as first argument to #[signal(name, Event, filter)] attribute."));
            };
            let Some(name) = name.path().get_ident() else {
                let span = name.span();
                return Err(syn::Error::new(span, "Expected ident as first argument to #[signal(name, Event, filter)] attribute."));
            };
            let syn::NestedMeta::Meta(event) = signal_cfg[1] else {
                let span = signal_cfg[1].span();
                return Err(syn::Error::new(span, "Expected type path as second argument to #[signal(name, Event, filter)] attribute."));
            };
            let syn::NestedMeta::Meta(filter) = signal_cfg[2] else {
                let span = signal_cfg[2].span();
                return Err(syn::Error::new(span, "Expected ident as third argument to #[signal(name, Event, filter)] attribute."));
            };
            let Some(filter) = filter.path().get_ident() else {
                let span = filter.span();
                return Err(syn::Error::new(span, "Expected ident as third argument to #[signal(name, Event, filter)] attribute."));
            };
            let event = event.path();
            connect_body = quote! {
                #connect_body
                pub fn #name<
                    C: ::bevy::prelude::Component,
                    F: Fn(&mut #core::build::ConnectionBuilder<C, #event>)
                >(
                    &self,
                    world: &mut ::bevy::prelude::World,
                    source: ::bevy::prelude::Entity,
                    build: F,
                    // target: #core::build::ConnectionTo<C, #event>
                ) {
                    let mut builder = #core::build::ConnectionBuilder::<C, #event>::default();
                    build(&mut builder);
                    if let Some(target) = builder.build() {
                        target
                            .filter(|e| e.#filter())
                            .from(source)
                            .write(world)
                    } else {
                        ::bevy::prelude::error!("Unable to create connection");
                    }
                }
            }
        }
    }
    Ok(connect_body)
}

fn prepare_extends_descriptor(
    ident: &syn::Ident,
    attrs: &Vec<syn::Attribute>,
) -> syn::Result<TokenStream> {
    let core = core_path();
    let this_str = ident.to_string();
    let this_mod = format_ident!("{}_widget_descriptor", this_str.to_lowercase());
    let default_descriptor = quote! {
        impl ::std::ops::Deref for #this_mod::Descriptor {
            type Target = #core::eml::build::DefaultDescriptor;
            fn deref(&self) -> &#core::eml::build::DefaultDescriptor {
                let instance = #core::eml::build::DefaultDescriptor::get_instance();
                instance
            }
        }
    };
    let Some(extends) = Extend::from_attributes(attrs)? else {
        return Ok(default_descriptor)
    };
    let Some(descriptor) = extends.descriptor() else {
        return Ok(default_descriptor)
    };

    let extends = descriptor.to_string();
    let extends = format_ident!("{}WidgetExtension", capitalize(&extends));
    let derive = quote! {
        impl ::std::ops::Deref for #this_mod::Descriptor {
            type Target = <#core::Widgets as #extends>::Descriptor;
            fn deref(&self) -> &<#core::Widgets as #extends>::Descriptor {
                let instance = <#core::Widgets as #extends>::Descriptor::get_instance();
                instance
            }
        }
    };
    Ok(derive)
}

fn prepare_extends_aliases(attrs: &Vec<syn::Attribute>) -> syn::Result<TokenStream> {
    let core = core_path();
    let default_styles = quote! {};
    let Some(extends) = Extend::from_attributes(attrs)? else {
        return Ok(default_styles)
    };
    let Some(styles) = extends.styles() else {
        return Ok(default_styles)
    };
    let ext = styles.to_string();
    let ext = format_ident!("{}WidgetExtension", capitalize(&ext));
    Ok(quote! {
        fn aliases() -> &'static [&'static str] {
            unsafe {
                static mut ALIASES: Vec<&'static str> = vec![];
                static ONCE: ::std::sync::Once = ::std::sync::Once::new();
                ONCE.call_once(|| {
                    let instance = <#core::Widgets as #ext>::Descriptor::get_instance();
                    let builder = instance.get_builder();
                    ALIASES.extend(builder.names().map(|t| *t));
                    ALIASES.extend(builder.aliases().map(|t| *t));
                });
                &ALIASES
            }
         }
    })
}

fn parse_styles(ident: &syn::Ident, attrs: &Vec<syn::Attribute>) -> syn::Result<TokenStream> {
    let mut styles = "".to_string();
    let element = ident.to_string();
    for attr in attrs.iter().filter(|a| a.path.is_ident("style")) {
        let span = attr.span();
        let Ok(meta) = attr.parse_meta() else {
            return Err(syn::Error::new(span,  "Invalid syntax fo #[signal(name, Event, filter)] attribute."));
        };
        let syn::Meta::List(style) = meta else {
            return Err(syn::Error::new(span, "Invalid syntax fo #[signal(name, Event, filter)] attribute."));    
        };
        let style: Vec<_> = style.nested.iter().collect();
        if style.len() == 1 {
            let syn::NestedMeta::Lit(syn::Lit::Str(prop)) = style[0] else {
                let span = style[0].span();
                return Err(syn::Error::new(span, "Non-literal token ident as first argument to #[widget(\"prop: value\")] attribute."));
            };
            let style = prop.value();
            styles += &format!("{element}: {{ {style} }}\n")
        } else {
            let syn::NestedMeta::Lit(syn::Lit::Str(selector)) = style[0] else {
                let span = style[0].span();
                return Err(syn::Error::new(span, "Non-literal token in #[widget(\"selector\", \"prop: value\")] attribute."));
            };
            let selector = selector.value();
            let mut props = "".to_string();
            for prop in style.iter().skip(1) {
                let syn::NestedMeta::Lit(syn::Lit::Str(prop)) = prop else {
                    let span = style[0].span();
                    return Err(syn::Error::new(span, "Non-literal token in #[widget(\"selector\", \"prop: value\")] attribute."));
                };
                props += &prop.value();
                props += "; "
            }
            styles += &format!("{selector} {{ {props} }}");
        }

        // let Ok(style) = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated.parse2(attrs) else {
        //     return Err(syn::Error::new(span, "#[style] macro attributes should string literals: #[widget(\"font: bold\")]"));
        // };
        // if style.len() == 1 {
        //     let style = style[0].value();
        //     styles += &format!("{element}: {{ {style} }}\n")
        // } else {
        //     let selector = style[0].value();
        //     let mut props = "".to_string();
        //     for prop in style.iter().skip(1) {
        //         props += &prop.value();
        //         props += "; ";
        //     }
        //     styles += &format!("{selector} {{ {props} }}\n");
        // }
    }

    if styles.len() > 0 {
        // let styles = format!("{element}: {{ {styles} }}");
        return Ok(quote! {
            fn styles() -> &'static str {
                #styles
            }
        });
    } else {
        return Ok(quote! {
            fn styles() -> &'static str {
                ""
            }
        });
    }
}

#[proc_macro_attribute]
pub fn widget(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as syn::ItemFn);
    let core = core_path();
    let fn_ident = ast.sig.ident;
    let fn_args = ast.sig.inputs;
    let fn_body = ast.block;
    let alias = fn_ident.to_string();
    let mod_descriptor = format_ident!("{}_widget_descriptor", &alias);
    let extension = format_ident!("{}WidgetExtension", capitalize(&alias));
    let (_doclines, docs) = parse_docs(&ast.attrs);

    let connect_signals = match parse_signals(&ast.attrs) {
        Ok(tokens) => tokens,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };
    let extends_decl = match prepare_extends_descriptor(&fn_ident, &ast.attrs) {
        Ok(tokens) => tokens,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };
    let aliases_decl = match prepare_extends_aliases(&ast.attrs) {
        Ok(tokens) => tokens,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };
    let styles_decl = match parse_styles(&fn_ident, &ast.attrs) {
        Ok(tokens) => tokens,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    proc_macro::TokenStream::from(quote! {

        mod #mod_descriptor {
        use super::*;

        #[derive(Component)]
        #[allow(non_camel_case_types)]
        pub struct #fn_ident;

        pub struct Descriptor;
        impl Descriptor {
            pub fn get_instance() -> &'static Descriptor {
                &&Descriptor
            }

            pub fn get_builder(&self) -> #core::build::ElementBuilder {
                #fn_ident::as_builder()
            }
            #styles_decl
            #connect_signals
        }

        impl #core::build::Widget for #fn_ident {
            fn names() -> &'static [&'static str] {
                &[#alias]
            }
            #aliases_decl
        }

        impl #core::build::WidgetBuilder for #fn_ident {
            #styles_decl
            fn construct(#fn_args) {
                #fn_body
            }
        }

        pub trait #extension {
            type Descriptor;
            #docs
            fn #fn_ident() -> Descriptor {
                Descriptor
            }

            fn descriptor() -> Self::Descriptor;
        }

        impl #extension for #core::Widgets {
            type Descriptor = Descriptor;
            fn descriptor() -> Self::Descriptor {
                Descriptor
            }
        }
        }
        pub use #mod_descriptor::#extension;
        #[allow(non_camel_case_types)]
        pub (crate) type #fn_ident = #mod_descriptor::#fn_ident;

        #extends_decl



    })
}
